from datetime import datetime
from enum import Enum
from typing import Any

from pydantic import BaseModel, Field


class EventType(str, Enum):
    utterance = "utterance"
    action = "action"
    feedback = "feedback"
    import_ = "import"


class EventAppendRequest(BaseModel):
    content: str = Field(min_length=1)
    conversation_id: str | None = None
    event_type: EventType = EventType.utterance
    ts: datetime | None = None
    agent: str | None = None
    meta: dict[str, Any] | None = None


class EventAppendResponse(BaseModel):
    event_id: str
    trace_id: str
    conversation_id: str
    accepted: bool = True


class EventBatchAppendRequest(BaseModel):
    events: list[EventAppendRequest] = Field(min_length=1)


class EventBatchAppendResponse(BaseModel):
    items: list[EventAppendResponse]
