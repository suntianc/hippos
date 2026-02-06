from __future__ import annotations

from typing import Any, Protocol

from app.schemas.common import AuditItem, FactItem, RecallItem, RelationItem
from app.schemas.events import EventAppendRequest, EventAppendResponse


class MemoryRepository(Protocol):
    def append_event(
        self,
        payload: EventAppendRequest,
        *,
        tenant_id: str,
        user_id: str,
        agent_id: str,
    ) -> EventAppendResponse: ...

    def list_facts(self, scope: str | None, *, tenant_id: str, user_id: str) -> list[FactItem]: ...

    def query_memory(
        self,
        query_text: str,
        top_k: int,
        *,
        tenant_id: str,
        user_id: str,
    ) -> tuple[list[FactItem], list[RecallItem], list[RelationItem]]: ...

    def override_fact(
        self,
        fact_id: str,
        fact_value: Any,
        scope: str | None,
        trace_id: str,
        *,
        tenant_id: str,
        user_id: str,
    ) -> bool: ...

    def revoke_fact(
        self,
        fact_id: str,
        trace_id: str,
        reason: str | None,
        *,
        tenant_id: str,
        user_id: str,
    ) -> bool: ...

    def delete_recall(self, recall_id: str, trace_id: str, *, tenant_id: str, user_id: str) -> bool: ...

    def get_audit_trace(self, trace_id: str) -> list[AuditItem]: ...
