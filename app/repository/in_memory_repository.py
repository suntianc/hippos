from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Any
from uuid import uuid4

from app.schemas.common import AuditItem, FactItem, RecallItem, RelationItem
from app.schemas.events import EventAppendRequest, EventAppendResponse


@dataclass
class StoredEvent:
    id: str
    trace_id: str
    conversation_id: str
    tenant_id: str
    user_id: str
    agent_id: str
    content: str
    event_type: str
    ts: datetime
    meta: dict[str, Any]


class InMemoryRepository:
    def __init__(self) -> None:
        self.events: dict[str, StoredEvent] = {}
        self.facts: dict[str, FactItem] = {}
        self.recalls: dict[str, RecallItem] = {}
        self.relations: dict[str, RelationItem] = {}
        self.audits_by_trace: dict[str, list[AuditItem]] = {}

    def append_event(
        self,
        payload: EventAppendRequest,
        *,
        tenant_id: str,
        user_id: str,
        agent_id: str,
    ) -> EventAppendResponse:
        event_id = f"event:{uuid4().hex}"
        trace_id = f"trace:{uuid4().hex}"
        conversation_id = payload.conversation_id or f"conv:{uuid4().hex}"
        ts = payload.ts or datetime.now(timezone.utc)
        stored = StoredEvent(
            id=event_id,
            trace_id=trace_id,
            conversation_id=conversation_id,
            tenant_id=tenant_id,
            user_id=user_id,
            agent_id=payload.agent or agent_id,
            content=payload.content,
            event_type=payload.event_type.value,
            ts=ts,
            meta=payload.meta or {},
        )
        self.events[event_id] = stored
        self._append_audit(
            trace_id,
            AuditItem(
                action="propose",
                target_type="event",
                target_id=event_id,
                reason="event accepted",
                operator="agent",
                ts=datetime.now(timezone.utc),
            ),
        )
        return EventAppendResponse(event_id=event_id, trace_id=trace_id, conversation_id=conversation_id)

    def list_facts(self, scope: str | None, *, tenant_id: str, user_id: str) -> list[FactItem]:
        del tenant_id, user_id
        values = list(self.facts.values())
        if scope is None:
            return values
        return [x for x in values if x.scope == scope]

    def query_memory(
        self,
        query_text: str,
        top_k: int,
        *,
        tenant_id: str,
        user_id: str,
    ) -> tuple[list[FactItem], list[RecallItem], list[RelationItem]]:
        del query_text, tenant_id, user_id
        facts = list(self.facts.values())[:top_k]
        recalls = list(self.recalls.values())[:top_k]
        relations = list(self.relations.values())[:top_k]
        return facts, recalls, relations

    def override_fact(
        self,
        fact_id: str,
        fact_value: Any,
        scope: str | None,
        trace_id: str,
        *,
        tenant_id: str,
        user_id: str,
    ) -> bool:
        del tenant_id, user_id
        fact = self.facts.get(fact_id)
        if fact is None:
            return False
        fact.fact_value = fact_value
        fact.scope = scope if scope is not None else fact.scope
        fact.version += 1
        fact.status = "active"
        self._append_audit(
            trace_id,
            AuditItem(
                action="override",
                target_type="fact",
                target_id=fact_id,
                reason="fact overridden",
                operator="user",
                ts=datetime.now(timezone.utc),
            ),
        )
        return True

    def revoke_fact(
        self,
        fact_id: str,
        trace_id: str,
        reason: str | None,
        *,
        tenant_id: str,
        user_id: str,
    ) -> bool:
        del tenant_id, user_id
        fact = self.facts.get(fact_id)
        if fact is None:
            return False
        fact.status = "revoked"
        self._append_audit(
            trace_id,
            AuditItem(
                action="revoke",
                target_type="fact",
                target_id=fact_id,
                reason=reason or "fact revoked",
                operator="user",
                ts=datetime.now(timezone.utc),
            ),
        )
        return True

    def delete_recall(self, recall_id: str, trace_id: str, *, tenant_id: str, user_id: str) -> bool:
        del tenant_id, user_id
        if recall_id not in self.recalls:
            return False
        del self.recalls[recall_id]
        self._append_audit(
            trace_id,
            AuditItem(
                action="delete",
                target_type="recall",
                target_id=recall_id,
                reason="recall deleted",
                operator="user",
                ts=datetime.now(timezone.utc),
            ),
        )
        return True

    def get_audit_trace(self, trace_id: str) -> list[AuditItem]:
        return self.audits_by_trace.get(trace_id, [])

    def _append_audit(self, trace_id: str, item: AuditItem) -> None:
        self.audits_by_trace.setdefault(trace_id, []).append(item)
