# Desktop Rust Layer

`tracker.rs` is the platform boundary.

The current starter calls `active-win-pos-rs` and prints the active app/title every 3 seconds. Next implementation step:

1. Add a `TrackerObservation` struct.
2. Track the current app and the previous app.
3. When the app changes, close the previous session.
4. Write a `UsageEvent` to SQLite.
5. Run the rule engine on the session duration.
6. Suppress repeated alerts using `notification_cooldown_minutes`.
7. Send the message through Tauri notification plugin.

Do not put threshold logic inside `tracker.rs`.
