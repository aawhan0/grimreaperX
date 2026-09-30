use crate::models::UsageEvent;
use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AppUsageSummary {
  pub app_name: String,
  pub seconds: i64,
}

pub fn init(conn: &Connection) -> Result<()> {
    conn.execute_batch(r#"
      CREATE TABLE IF NOT EXISTS device(id TEXT PRIMARY KEY, platform TEXT NOT NULL, user_id TEXT);
      CREATE TABLE IF NOT EXISTS usage_event(
        id TEXT PRIMARY KEY, device_id TEXT NOT NULL, app_name TEXT NOT NULL,
        category TEXT NOT NULL, start_ts TEXT NOT NULL, end_ts TEXT NOT NULL
      );
      CREATE TABLE IF NOT EXISTS threat_log(
        id INTEGER PRIMARY KEY AUTOINCREMENT, usage_event_id TEXT, tier TEXT NOT NULL,
        message TEXT NOT NULL, delivered_ts TEXT NOT NULL
      );
      CREATE TABLE IF NOT EXISTS app_rule(
        app_name TEXT PRIMARY KEY, threshold_minutes INTEGER NOT NULL, enabled INTEGER NOT NULL
      );
      CREATE TABLE IF NOT EXISTS app_setting(
        key TEXT PRIMARY KEY, value INTEGER NOT NULL
      );
      CREATE INDEX IF NOT EXISTS idx_usage_app_time ON usage_event(app_name, start_ts);
      CREATE INDEX IF NOT EXISTS idx_threat_delivered_time ON threat_log(delivered_ts);
    "#)
}

pub fn get_threshold_minutes(conn: &Connection) -> Result<i64> {
    conn.query_row(
        "SELECT value FROM app_setting WHERE key = 'threshold_minutes'",
        [],
        |row| row.get(0),
    )
    .optional()
    .map(|value| value.unwrap_or(20))
}

pub fn set_threshold_minutes(conn: &Connection, minutes: i64) -> Result<()> {
    conn.execute(
        "INSERT INTO app_setting(key, value) VALUES('threshold_minutes', ?1) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [minutes],
    )?;
    Ok(())
}

pub fn insert_usage(conn: &Connection, event: &UsageEvent) -> Result<()> {
    conn.execute(
      "INSERT OR REPLACE INTO usage_event(id,device_id,app_name,category,start_ts,end_ts) VALUES(?,?,?,?,?,?)",
      params![event.id.to_string(), event.device_id, event.app_name, event.category, event.start_ts.to_rfc3339(), event.end_ts.to_rfc3339()]
    )?;
    Ok(())
}

  pub fn insert_threat(
    conn: &Connection,
    usage_event_id: &str,
    tier: &str,
    message: &str,
    delivered_ts: chrono::DateTime<chrono::Utc>,
  ) -> Result<()> {
    conn.execute(
      "INSERT INTO threat_log(usage_event_id, tier, message, delivered_ts) VALUES(?, ?, ?, ?)",
      params![usage_event_id, tier, message, delivered_ts.to_rfc3339()],
    )?;
    Ok(())
  }

  pub fn today_usage_seconds(conn: &Connection) -> Result<i64> {
    conn.query_row(
      "SELECT COALESCE(CAST(SUM(MAX(0, (julianday(end_ts) - julianday(start_ts)) * 86400)) AS INTEGER), 0) \
       FROM usage_event WHERE date(start_ts, 'localtime') = date('now', 'localtime')",
      [],
      |row| row.get(0),
    )
  }

  pub fn top_apps_today(conn: &Connection) -> Result<Vec<AppUsageSummary>> {
    let mut statement = conn.prepare(
      "SELECT app_name, CAST(SUM(MAX(0, (julianday(end_ts) - julianday(start_ts)) * 86400)) AS INTEGER) AS seconds \
       FROM usage_event WHERE date(start_ts, 'localtime') = date('now', 'localtime') \
       GROUP BY app_name ORDER BY seconds DESC LIMIT 5",
    )?;
    let rows = statement.query_map([], |row| {
      Ok(AppUsageSummary {
        app_name: row.get(0)?,
        seconds: row.get(1)?,
      })
    })?;
    rows.collect()
  }

  pub fn threats_today(conn: &Connection) -> Result<i64> {
    conn.query_row(
      "SELECT COUNT(*) FROM threat_log WHERE date(delivered_ts, 'localtime') = date('now', 'localtime')",
      [],
      |row| row.get(0),
    )
  }

  #[cfg(test)]
  mod tests {
    use super::*;

    #[test]
    fn threshold_defaults_to_twenty_minutes() {
      let conn = Connection::open_in_memory().unwrap();
      init(&conn).unwrap();

      assert_eq!(get_threshold_minutes(&conn).unwrap(), 20);
    }

    #[test]
    fn threshold_setting_persists() {
      let conn = Connection::open_in_memory().unwrap();
      init(&conn).unwrap();
      set_threshold_minutes(&conn, 35).unwrap();

      assert_eq!(get_threshold_minutes(&conn).unwrap(), 35);
    }

      #[test]
      fn dashboard_aggregates_usage_and_threats() {
        let conn = Connection::open_in_memory().unwrap();
        init(&conn).unwrap();
        let ended_at = chrono::Utc::now();
        let event = UsageEvent {
          id: uuid::Uuid::new_v4(),
          device_id: "desktop".into(),
          app_name: "Code".into(),
          category: "development".into(),
          start_ts: ended_at - chrono::Duration::minutes(3),
          end_ts: ended_at,
        };
        insert_usage(&conn, &event).unwrap();
        insert_threat(
          &conn,
          &event.id.to_string(),
          "mild",
          "test message",
          ended_at,
        )
        .unwrap();

        assert!(today_usage_seconds(&conn).unwrap() >= 179);
        assert_eq!(top_apps_today(&conn).unwrap()[0].app_name, "Code");
        assert_eq!(threats_today(&conn).unwrap(), 1);
      }
  }
