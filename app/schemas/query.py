from pydantic import BaseModel, Field

from app.schemas.common import FactItem, RecallItem, RelationItem


class MemoryQueryRequest(BaseModel):
    query_text: str = Field(min_length=1)
    scope: str | None = None
    top_k: int = Field(default=10, ge=1, le=100)
    hops: int = Field(default=1, ge=1, le=4)
    conversation_id: str | None = None


class MemoryQueryResponse(BaseModel):
    facts: list[FactItem]
    recalls: list[RecallItem]
    relations: list[RelationItem]
    trace_id: str
