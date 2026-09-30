mod models;
mod rule_engine;
mod store;
mod threats;
mod tracker;

use rusqlite::Connection;
use serde::Serialize;
use std::{
    fs,
    sync::{
        atomic::{AtomicBool, AtomicI64, Ordering},
        Arc, Mutex,
    },
    thread,
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager, State,
};

struct AppState {
    db: Mutex<Connection>,
    threshold_minutes: Arc<AtomicI64>,
    status: Arc<Mutex<tracker::TrackerStatus>>,
}

#[derive(Debug, Serialize)]
struct DashboardSummary {
    today_total_seconds: i64,
    top_apps: Vec<store::AppUsageSummary>,
    threat_count: i64,
    current_tier: &'static str,
}

#[tauri::command]
fn health() -> String { "grimreaperX desktop agent is alive".into() }

#[tauri::command]
fn get_tracker_status(state: State<'_, AppState>) -> Result<tracker::TrackerStatus, String> {
    let mut snapshot = state.status.lock().map_err(|error| error.to_string())?.clone();
    snapshot.threshold_minutes = state.threshold_minutes.load(Ordering::Relaxed);
    Ok(snapshot)
}

#[tauri::command]
fn set_threshold_minutes(minutes: i64, state: State<'_, AppState>) -> Result<(), String> {
    if !(1..=1440).contains(&minutes) {
        return Err("Threshold must be between 1 and 1440 minutes".into());
    }

    let conn = state.db.lock().map_err(|error| error.to_string())?;
    store::set_threshold_minutes(&conn, minutes).map_err(|error| error.to_string())?;
    state.threshold_minutes.store(minutes, Ordering::Relaxed);
    if let Ok(mut snapshot) = state.status.lock() {
        snapshot.threshold_minutes = minutes;
    }
    Ok(())
}

#[tauri::command]
fn get_dashboard_summary(state: State<'_, AppState>) -> Result<DashboardSummary, String> {
    let snapshot = state.status.lock().map_err(|error| error.to_string())?.clone();
    let conn = state.db.lock().map_err(|error| error.to_string())?;
    let mut today_total_seconds = store::today_usage_seconds(&conn).map_err(|error| error.to_string())?;
    let mut top_apps = store::top_apps_today(&conn).map_err(|error| error.to_string())?;
    let threat_count = store::threats_today(&conn).map_err(|error| error.to_string())?;

    let active_session_started_today = snapshot
        .session_started_at
        .map(|started_at| started_at.with_timezone(&chrono::Local).date_naive() == chrono::Local::now().date_naive())
        .unwrap_or(false);
    if active_session_started_today {
        if let Some(app_name) = snapshot.current_app {
            today_total_seconds += snapshot.elapsed_seconds;
            if let Some(app) = top_apps.iter_mut().find(|app| app.app_name == app_name) {
                app.seconds += snapshot.elapsed_seconds;
            } else {
                top_apps.push(store::AppUsageSummary {
                    app_name,
                    seconds: snapshot.elapsed_seconds,
                });
            }
            top_apps.sort_by_key(|app| std::cmp::Reverse(app.seconds));
            top_apps.truncate(5);
        }
    }

    let tier = crate::rule_engine::evaluate(snapshot.elapsed_seconds / 60, snapshot.threshold_minutes);
    let current_tier = match tier {
        models::Tier::Normal => "normal",
        models::Tier::Mild => "mild",
        models::Tier::Serious => "serious",
        models::Tier::Unhinged => "unhinged",
    };

    Ok(DashboardSummary {
        today_total_seconds,
        top_apps,
        threat_count,
        current_tier,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let tracking = Arc::new(AtomicBool::new(true));
            let path = app.path().app_data_dir()?.join("grimreaperx.sqlite");
            fs::create_dir_all(path.parent().unwrap())?;
            let conn = Connection::open(&path)?;
            store::init(&conn)?;
            let threshold = store::get_threshold_minutes(&conn)?;
            let threshold_minutes = Arc::new(AtomicI64::new(threshold));
            let status = Arc::new(Mutex::new(tracker::TrackerStatus {
                current_app: None,
                elapsed_seconds: 0,
                session_started_at: None,
                is_tracking: true,
                threshold_minutes: threshold,
            }));
            app.manage(AppState {
                db: Mutex::new(conn),
                threshold_minutes: threshold_minutes.clone(),
                status: status.clone(),
            });

            let show = MenuItem::with_id(app, "show", "Show dashboard", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let pause = MenuItem::with_id(app, "pause", "Pause tracking", true, None::<&str>)?;
            let resume = MenuItem::with_id(app, "resume", "Resume tracking", false, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit grimreaperX", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &settings, &pause, &resume, &quit])?;

            let pause_tracking = tracking.clone();
            let resume_tracking = tracking.clone();
            let mut tray = TrayIconBuilder::new().menu(&menu).tooltip("grimreaperX");
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "settings" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let _ = window.emit("open-settings", ());
                        }
                    }
                    "pause" => {
                        pause_tracking.store(false, std::sync::atomic::Ordering::Relaxed);
                        let _ = pause.set_enabled(false);
                        let _ = resume.set_enabled(true);
                    }
                    "resume" => {
                        resume_tracking.store(true, std::sync::atomic::Ordering::Relaxed);
                        let _ = resume.set_enabled(false);
                        let _ = pause.set_enabled(true);
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            let loop_path = path.clone();
            let loop_tracking = tracking;
            let loop_threshold = threshold_minutes;
            let loop_status = status;
            thread::spawn(move || {
                let conn = Connection::open(loop_path).expect("demo loop should open sqlite connection");
                tracker::run_demo_loop(&conn, &loop_tracking, &loop_threshold, &loop_status);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            health,
            get_tracker_status,
            set_threshold_minutes,
            get_dashboard_summary
        ])
        .run(tauri::generate_context!())
        .expect("error while running grimreaperX");
}
