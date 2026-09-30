# Week 1 — Exact Implementation Order

1. **Tracker observation**: poll every 3 seconds.
2. **Session aggregator**: same app + process continues the current session; app switch closes it.
3. **SQLite**: insert closed sessions only.
4. **Rule evaluation**: evaluate closed session duration; for continuous alerts, also maintain an in-memory current-session timer.
5. **Cooldown**: store last delivered time per app/tier to avoid notification spam.
6. **Threat generation**: select one template from the matching tier and substitute `{app}`.
7. **Notification**: deliver via Tauri notification plugin.
8. **Demo mode**: add a debug threshold of 1 minute so the complete loop can be demonstrated without waiting 20 minutes.

## Important design decision

Do not block the tracker while generating a threat. Notification generation/delivery should happen after the event has been persisted.

## Cumulative mode

For cumulative rules, query today's usage rows for the app and sum durations. Use UTC timestamps internally and localize only in the UI.
