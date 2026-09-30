# Development Checklist

## Week 1
- [ ] Tauri starts
- [ ] Active window polling works on Windows
- [ ] App session aggregation works
- [ ] SQLite schema created
- [ ] Events inserted
- [ ] Rule engine tests pass
- [ ] Threat template loading works
- [ ] Notification delivered once per cooldown

## Week 2
- [x] Tray menu
- [x] Settings UI
- [x] Dashboard cards
- [x] Android Usage Access flow
- [x] Android foreground service
- [x] Android rule engine and escalation
- [x] Android notifications
- [x] Shared event serialization
- [ ] Desktop/Android end-to-end demo

## Week 3
- [ ] FastAPI
- [ ] PostgreSQL
- [ ] JWT
- [ ] Batch sync
- [ ] Idempotency
- [ ] Rate limiting
- [ ] Security review
- [ ] Packaging

## Demo script
1. Set threshold to 1 minute.
2. Open a tracked application.
3. Keep it active for >1 minute.
4. Show SQLite usage event.
5. Show randomized threat notification.
6. Increase duration / simulate it to demonstrate escalation.
