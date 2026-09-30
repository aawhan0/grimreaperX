import os
from datetime import datetime, timedelta, timezone
from uuid import uuid4

from fastapi.testclient import TestClient

os.environ["RATE_LIMIT_REQUESTS"] = "2"
os.environ["RATE_LIMIT_WINDOW_SECONDS"] = "60"
os.environ["MAX_REQUEST_BYTES"] = "256"

from backend.app.main import app, _rate_buckets  # noqa: E402

client = TestClient(app)


def event(**overrides):
    payload = {
        "id": str(uuid4()),
        "device_id": "device-1",
        "app_name": "Code",
        "category": "development",
        "start_ts": "2026-09-30T10:00:00Z",
        "end_ts": "2026-09-30T10:10:00Z",
    }
    payload.update(overrides)
    return payload


def setup_function():
    _rate_buckets.clear()


def test_health_is_public():
    response = client.get("/health")
    assert response.status_code == 200
    assert response.json() == {"status": "ok"}


def test_rejects_reverse_time_interval():
    response = client.post(
        "/v1/usage-events",
        json=event(end_ts="2026-09-30T09:59:59Z"),
    )
    assert response.status_code == 422


def test_rejects_blank_identifier():
    response = client.post("/v1/usage-events", json=event(device_id="   "))
    assert response.status_code == 422


def test_rejects_naive_timestamps():
    response = client.post(
        "/v1/usage-events",
        json=event(start_ts="2026-09-30T10:00:00", end_ts="2026-09-30T10:10:00"),
    )
    assert response.status_code == 422


def test_rate_limit_returns_429_with_retry_after():
    assert client.post("/v1/usage-events", json=event()).status_code == 202
    assert client.post("/v1/usage-events", json=event()).status_code == 202
    response = client.post("/v1/usage-events", json=event())
    assert response.status_code == 429
    assert int(response.headers["Retry-After"]) > 0


def test_oversized_request_is_rejected():
    response = client.post(
        "/v1/usage-events",
        content=b"{" + b"a" * 300 + b"}",
        headers={"Content-Type": "application/json", "Content-Length": "302"},
    )
    assert response.status_code == 413
