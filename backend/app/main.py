from fastapi import FastAPI
from pydantic import BaseModel, Field
from datetime import datetime
from uuid import UUID

app = FastAPI(title="grimreaperX API", version="0.1.0")

class UsageEvent(BaseModel):
    id: UUID
    device_id: str = Field(min_length=1)
    app_name: str = Field(min_length=1)
    category: str
    start_ts: datetime
    end_ts: datetime

@app.get("/health")
def health():
    return {"status": "ok"}

@app.post("/v1/usage-events", status_code=202)
def ingest_usage(event: UsageEvent):
    # Week 3: persist to PostgreSQL and enforce idempotency on event.id.
    return {"accepted": True, "event_id": str(event.id)}
