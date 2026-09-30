# Architecture Notes

## Core interfaces

### Tracker
Produces raw active-window observations.

### Normalizer
Turns platform-specific observations into `UsageEvent`.

### Store
SQLite locally; PostgreSQL remotely later.

### Rule Engine
Pure business logic. It should not know whether the event came from Windows or Android.

### Threat Generator
Static templates first; Ollama optional.

### Delivery
Platform-specific notifications.

## Event contract

```json
{
  "id": "uuid",
  "device_id": "device-123",
  "app_name": "Chrome",
  "category": "browser",
  "start_ts": "2026-09-29T18:00:00Z",
  "end_ts": "2026-09-29T18:25:00Z"
}
```

## Tier policy

- `normal`: under threshold
- `mild`: 0–5 minutes over
- `serious`: 5–30 minutes over
- `unhinged`: 30+ minutes over

Keep the policy configurable rather than hard-coded into UI code.

## iOS limitation

iOS is intentionally not implemented in v1. Apple's Screen Time APIs have entitlement and product restrictions that make a general-purpose background tracker materially different from the Windows/Android approach.
