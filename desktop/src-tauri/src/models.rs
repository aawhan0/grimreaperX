use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    pub id: Uuid,
    pub device_id: String,
    pub app_name: String,
    pub category: String,
    pub start_ts: DateTime<Utc>,
    pub end_ts: DateTime<Utc>,
}

impl UsageEvent {
    pub fn duration_minutes(&self) -> i64 {
        (self.end_ts - self.start_ts).num_minutes().max(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    Normal,
    Mild,
    Serious,
    Unhinged,
}

#[derive(Debug, Clone)]
pub struct AppRule {
    pub app_name: String,
    pub threshold_minutes: i64,
    pub enabled: bool,
}
