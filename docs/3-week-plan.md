# grimreaperX — 3 Week Build Plan

## Week 1 — Windows MVP

### Day 1 — Workspace + architecture
- Install Rust, Node, Tauri prerequisites.
- Run the Tauri starter.
- Read `desktop/src-tauri/src/models.rs` and `rule_engine.rs`.
- Define the event lifecycle.

**Deliverable:** desktop app launches and project structure is understood.

### Day 2 — Active window tracker
- Integrate/update `active-win-pos-rs`.
- Poll every 2–5 seconds.
- Capture process name, title, timestamp.
- Ignore desktop/idle windows.

**Deliverable:** terminal/log output shows the active process changing.

### Day 3 — Usage normalization
- Convert raw windows into `UsageEvent` records.
- Merge consecutive observations from the same app.
- Add category mapping.

**Deliverable:** clean app sessions instead of noisy polling events.

### Day 4 — SQLite
- Create local database.
- Store devices, usage events, app rules, threat logs.
- Add indexes for app + timestamp queries.

**Deliverable:** usage survives app restart.

### Day 5 — Rule engine
- Implement continuous threshold.
- Implement cumulative threshold as a second strategy.
- Add escalation tiers.

**Deliverable:** deterministic unit tests for 0/19/20/25/50+ minutes.

### Day 6 — Threat generator + notifications
- Load JSON template bank.
- Randomize from the correct tier.
- Add desktop notification.

**Deliverable:** crossing a threshold produces a randomized notification.

### Day 7 — Week 1 demo
- Package the Windows app.
- Test Spotify/Chrome/VS Code/Discord or another set of apps.
- Fix false positives and duplicate notifications.

**Demo:** open an app, lower threshold to 1 minute, receive a threat.

---

## Week 2 — Escalation + Android

### Day 8 — Desktop UX
- Tray menu: pause/resume, settings, quit.
- Add current app/time-over display.

### Day 9 — Dashboard MVP
- Today total.
- Top apps.
- Threat count.
- Current escalation tier.

### Day 10 — Android permission flow
- Request `PACKAGE_USAGE_STATS` through system Settings.
- Explain why permission is needed.

### Day 11 — Android foreground service
- Read `UsageStatsManager` periodically.
- Normalize usage into the same JSON contract.

### Day 12 — Android rule engine
- Port rule behavior using Kotlin models.
- Add notification channels.

### Day 13 — Android escalation
- High-priority notifications.
- Prototype full-screen intent only for legitimate, user-initiated alarm-like UX; do not abuse it for ordinary reminders.

### Day 14 — Week 2 demo
- Desktop and Android both generate equivalent events.
- Same threat bank produces tiered messages.

---

## Week 3 — Sync + hardening

### Day 15 — FastAPI
- Devices endpoint.
- Usage event ingestion.
- Threat log ingestion.
- Health endpoint.

### Day 16 — PostgreSQL
- Apply schema from `backend/schema.sql`.
- Add indexes and constraints.

### Day 17 — Authentication
- Choose JWT access/refresh design.
- Hash passwords with Argon2/bcrypt if local auth is used.
- Never store raw tokens/passwords in SQLite.

### Day 18 — Offline sync
- Queue unsynced events locally.
- POST batches when online.
- Use event IDs/idempotency to prevent duplicates.

### Day 19 — Optional Ollama
- Add a feature flag.
- Static templates remain the zero-latency fallback.
- Constrain generated messages to short, fictional, non-violent content.

### Day 20 — Testing/security
- Unit + integration tests.
- Validate malformed events.
- Rate-limit threat delivery.
- Protect local API endpoints.
- Review permissions and logs.

### Day 21 — Release
- README screenshots/GIF.
- Architecture diagram.
- Demo script.
- GitHub release.
- Record known limitations: iOS, cloud sync maturity, platform-specific permissions.

## Definition of done

- [ ] Windows active-app tracking works.
- [ ] Usage is persisted locally.
- [ ] Thresholds are configurable.
- [ ] Duplicate notifications are suppressed.
- [ ] Threat tiers escalate.
- [ ] Android can produce normalized usage events.
- [ ] Backend accepts events.
- [ ] Tests cover the rule engine.
- [ ] No secrets are committed.
