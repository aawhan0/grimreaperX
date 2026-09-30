use active_win_pos_rs::get_active_window;
use chrono::{DateTime, Utc};
use notify_rust::Notification;
use rusqlite::Connection;
use serde::Serialize;
use std::{collections::HashMap, sync::atomic::{AtomicBool, Ordering}, thread, time::Duration};
use std::sync::{atomic::AtomicI64, Mutex};
use uuid::Uuid;

use crate::{
    models::UsageEvent,
    rule_engine::evaluate,
    store::{insert_threat, insert_usage},
    threats::{choose, default_templates, tier_key},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowSession {
    pub app_name: String,
    pub category: String,
    pub start_ts: DateTime<Utc>,
    pub end_ts: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrackerStatus {
    pub current_app: Option<String>,
    pub elapsed_seconds: i64,
    pub session_started_at: Option<DateTime<Utc>>,
    pub is_tracking: bool,
    pub threshold_minutes: i64,
}

impl WindowSession {
    pub fn new(app_name: &str, timestamp: DateTime<Utc>) -> Self {
        Self {
            app_name: app_name.to_string(),
            category: infer_category(app_name),
            start_ts: timestamp,
            end_ts: timestamp,
        }
    }
}

/// Platform adapter. Keep this module small: the rest of the application should
/// consume normalized app/process observations, not crate-specific structures.
pub fn sample_active_window() -> Option<(String, String)> {
    match get_active_window() {
        Ok(window) => {
            let app = normalize_app_name(&window.app_name);
            let title = window.title.trim().to_string();
            if app.is_empty() || is_ignored_window(&app) {
                None
            } else {
                Some((app, title))
            }
        }
        Err(_) => None,
    }
}

pub fn normalize_app_name(app_name: &str) -> String {
    let value = app_name.trim();
    let value = value.trim_matches(|c: char| c == '\0' || c == ' ');
    value.to_string()
}

pub fn infer_category(app_name: &str) -> String {
    let lower = app_name.to_ascii_lowercase();
    if lower.contains("chrome") || lower.contains("firefox") || lower.contains("edge") {
        "browser".to_string()
    } else if lower.contains("spotify") || lower.contains("discord") || lower.contains("slack") {
        "entertainment".to_string()
    } else if lower.contains("code") || lower.contains("visual studio") || lower.contains("sublime") {
        "development".to_string()
    } else if lower.contains("steam") || lower.contains("minecraft") {
        "gaming".to_string()
    } else {
        "general".to_string()
    }
}

pub fn is_ignored_window(app_name: &str) -> bool {
    let lower = app_name.to_ascii_lowercase();
    lower.is_empty()
        || lower.contains("desktop")
        || lower.contains("program manager")
        || lower.contains("idle")
        || lower == "explorer"
}

pub fn sync_session(current: &mut Option<WindowSession>, observed_app: &str, timestamp: DateTime<Utc>) -> Option<WindowSession> {
    let app_name = normalize_app_name(observed_app);
    if app_name.is_empty() || is_ignored_window(&app_name) {
        return current.take().map(|mut session| {
            session.end_ts = timestamp;
            session
        });
    }

    match current {
        Some(session) if session.app_name == app_name => {
            session.end_ts = timestamp;
            None
        }
        Some(session) => {
            let mut ended = session.clone();
            ended.end_ts = timestamp;
            *current = Some(WindowSession::new(&app_name, timestamp));
            Some(ended)
        }
        None => {
            *current = Some(WindowSession::new(&app_name, timestamp));
            None
        }
    }
}

pub fn run_demo_loop(
    conn: &Connection,
    tracking: &AtomicBool,
    threshold_minutes: &AtomicI64,
    status: &Mutex<TrackerStatus>,
) {
    let mut current: Option<WindowSession> = None;
    let mut last_delivered: HashMap<String, DateTime<Utc>> = HashMap::new();
    let mut paused_since = None;
    let templates = default_templates();

    loop {
        if !tracking.load(Ordering::Relaxed) {
            paused_since.get_or_insert_with(Utc::now);
            if let Ok(mut snapshot) = status.lock() {
                snapshot.is_tracking = false;
            }
            thread::sleep(Duration::from_secs(1));
            continue;
        }

        let now = Utc::now();
        if let Some(started_at) = paused_since.take() {
            if let Some(session) = current.as_mut() {
                let paused_duration = now - started_at;
                session.start_ts += paused_duration;
                session.end_ts += paused_duration;
            }
        }

        let observed_app = sample_active_window().map(|(app, _title)| app).unwrap_or_default();
        if let Some(session) = sync_session(&mut current, &observed_app, now) {
                let duration_minutes = (session.end_ts - session.start_ts).num_minutes().max(0);
                let event = UsageEvent {
                    id: Uuid::new_v4(),
                    device_id: "desktop".to_string(),
                    app_name: session.app_name.clone(),
                    category: session.category.clone(),
                    start_ts: session.start_ts,
                    end_ts: session.end_ts,
                };

                if let Err(err) = insert_usage(conn, &event) {
                    eprintln!("failed to persist usage event for {}: {err}", event.app_name);
                }

                let tier = evaluate(duration_minutes, threshold_minutes.load(Ordering::Relaxed));
                if tier != crate::models::Tier::Normal {
                    let key = format!("{}:{}", session.app_name, tier_key(tier));
                    let should_send = match last_delivered.get(&key) {
                        Some(last) => (now - *last).num_minutes() >= 15,
                        None => true,
                    };

                    if should_send {
                        if let Some(message) = choose(&templates, tier, &event.app_name) {
                            let delivered = Notification::new()
                                .summary("grimreaperX")
                                .body(&message)
                                .show()
                                .is_ok();
                            if delivered {
                                if let Err(err) = insert_threat(
                                    conn,
                                    &event.id.to_string(),
                                    tier_key(tier),
                                    &message,
                                    now,
                                ) {
                                    eprintln!("failed to persist threat delivery: {err}");
                                }
                                last_delivered.insert(key, now);
                            }
                        }
                    }
                }
        }

        if let Ok(mut snapshot) = status.lock() {
            snapshot.current_app = current.as_ref().map(|session| session.app_name.clone());
            snapshot.session_started_at = current.as_ref().map(|session| session.start_ts);
            snapshot.elapsed_seconds = current
                .as_ref()
                .map(|session| (now - session.start_ts).num_seconds().max(0))
                .unwrap_or(0);
            snapshot.is_tracking = true;
            snapshot.threshold_minutes = threshold_minutes.load(Ordering::Relaxed);
        }
        thread::sleep(Duration::from_secs(3));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn keeps_same_app_in_one_session() {
        let start = Utc.with_ymd_and_hms(2024, 1, 2, 9, 0, 0).unwrap();
        let later = Utc.with_ymd_and_hms(2024, 1, 2, 9, 3, 0).unwrap();

        let mut current: Option<WindowSession> = None;
        let ended = sync_session(&mut current, "Code", start);

        assert!(ended.is_none());
        let ended = sync_session(&mut current, "Code", later);
        assert!(ended.is_none());
        let session = current.expect("session should remain open");
        assert_eq!(session.app_name, "Code");
        assert_eq!(session.start_ts, start);
        assert_eq!(session.end_ts, later);
    }

    #[test]
    fn closes_session_when_app_changes() {
        let start = Utc.with_ymd_and_hms(2024, 1, 2, 9, 0, 0).unwrap();
        let switch = Utc.with_ymd_and_hms(2024, 1, 2, 9, 5, 0).unwrap();
        let new_app = Utc.with_ymd_and_hms(2024, 1, 2, 9, 6, 0).unwrap();

        let mut current: Option<WindowSession> = None;
        sync_session(&mut current, "Code", start);
        let ended = sync_session(&mut current, "Spotify", switch);

        let session = ended.expect("previous app should be closed when switching");
        assert_eq!(session.app_name, "Code");
        assert_eq!(session.end_ts, switch);

        let ended = sync_session(&mut current, "Spotify", new_app);
        assert!(ended.is_none());
        let session = current.expect("new app session should start");
        assert_eq!(session.app_name, "Spotify");
        assert_eq!(session.start_ts, switch);
    }

    #[test]
    fn paused_time_is_excluded_from_session_duration() {
        let start = Utc.with_ymd_and_hms(2024, 1, 2, 9, 0, 0).unwrap();
        let pause_end = Utc.with_ymd_and_hms(2024, 1, 2, 9, 10, 0).unwrap();
        let mut session = WindowSession::new("Code", start);
        session.end_ts = start + chrono::Duration::minutes(5);

        let active_duration_before_pause = session.end_ts - session.start_ts;
        let paused_duration = pause_end - (start + chrono::Duration::minutes(5));
        session.start_ts += paused_duration;
        session.end_ts += paused_duration;

        assert_eq!(session.end_ts - session.start_ts, active_duration_before_pause);
        assert_eq!(session.start_ts, start + chrono::Duration::minutes(5));
    }
}
