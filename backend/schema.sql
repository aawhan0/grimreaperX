CREATE TABLE IF NOT EXISTS device (
  id TEXT PRIMARY KEY,
  platform TEXT NOT NULL,
  user_id TEXT
);

CREATE TABLE IF NOT EXISTS usage_event (
  id UUID PRIMARY KEY,
  device_id TEXT NOT NULL REFERENCES device(id),
  app_name TEXT NOT NULL,
  category TEXT NOT NULL,
  start_ts TIMESTAMPTZ NOT NULL,
  end_ts TIMESTAMPTZ NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  CHECK (end_ts >= start_ts)
);

CREATE INDEX IF NOT EXISTS idx_usage_event_app_time ON usage_event(app_name, start_ts);

CREATE TABLE IF NOT EXISTS threat_log (
  id UUID PRIMARY KEY,
  usage_event_id UUID REFERENCES usage_event(id),
  tier TEXT NOT NULL,
  message TEXT NOT NULL,
  delivered_ts TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS app_rule (
  app_name TEXT PRIMARY KEY,
  threshold_minutes INTEGER NOT NULL CHECK (threshold_minutes > 0),
  enabled BOOLEAN NOT NULL DEFAULT TRUE
);
