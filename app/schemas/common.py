from datetime import datetime
from typing import Any

from pydantic import BaseModel, ConfigDict, Field


class GovernanceActionResponse(BaseModel):
    ok: bool
    action: str
    target_id: str
    trace_id: str


class AuditItem(BaseModel):
    action: str
    target_type: str
    target_id: str
    reason: str | None = None
    operator: str
    ts: datetime


class FactItem(BaseModel):
    id: str
    fact_key: str
    fact_value: Any
    scope: str | None = None
    version: int = 1
    status: str
    source_trace: str


class RecallItem(BaseModel):
    id: str
    text: str
    score: float
    event_ref: str


class RelationItem(BaseModel):
    model_config = ConfigDict(populate_by_name=True)

    id: str
    from_: str = Field(alias="from")
    to: str
    rel_type: str
    weight: float = 1.0
    scope: str | None = None


class AuditTraceResponse(BaseModel):
    trace_id: str
    items: list[AuditItem]


class FactOverrideRequest(BaseModel):
    fact_value: Any
    scope: str | None = None
    reason: str | None = None


class FactListResponse(BaseModel):
    facts: list[FactItem]
