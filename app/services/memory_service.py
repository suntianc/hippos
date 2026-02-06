from uuid import uuid4

from app.repository.factory import create_repository
from app.schemas.common import FactOverrideRequest, GovernanceActionResponse
from app.schemas.events import EventAppendRequest, EventAppendResponse
from app.schemas.query import MemoryQueryRequest, MemoryQueryResponse


class MemoryService:
    def __init__(self) -> None:
        self.repository = create_repository()

    def append_event(self, payload: EventAppendRequest, *, tenant_id: str, user_id: str, agent_id: str) -> EventAppendResponse:
        return self.repository.append_event(payload, tenant_id=tenant_id, user_id=user_id, agent_id=agent_id)

    def query_memory(self, payload: MemoryQueryRequest, *, tenant_id: str, user_id: str) -> MemoryQueryResponse:
        facts, recalls, relations = self.repository.query_memory(
            payload.query_text,
            payload.top_k,
            tenant_id=tenant_id,
            user_id=user_id,
        )
        return MemoryQueryResponse(
            facts=facts,
            recalls=recalls,
            relations=relations,
            trace_id=f"trace:{uuid4().hex}",
        )

    def list_facts(self, scope: str | None, *, tenant_id: str, user_id: str):
        return self.repository.list_facts(scope, tenant_id=tenant_id, user_id=user_id)

    def override_fact(
        self,
        fact_id: str,
        payload: FactOverrideRequest,
        *,
        tenant_id: str,
        user_id: str,
    ) -> GovernanceActionResponse:
        trace_id = f"trace:{uuid4().hex}"
        ok = self.repository.override_fact(
            fact_id,
            payload.fact_value,
            payload.scope,
            trace_id,
            tenant_id=tenant_id,
            user_id=user_id,
        )
        return GovernanceActionResponse(
            ok=ok,
            action="override",
            target_id=fact_id,
            trace_id=trace_id,
        )

    def revoke_fact(
        self,
        fact_id: str,
        reason: str | None,
        *,
        tenant_id: str,
        user_id: str,
    ) -> GovernanceActionResponse:
        trace_id = f"trace:{uuid4().hex}"
        ok = self.repository.revoke_fact(
            fact_id,
            trace_id,
            reason,
            tenant_id=tenant_id,
            user_id=user_id,
        )
        return GovernanceActionResponse(
            ok=ok,
            action="revoke",
            target_id=fact_id,
            trace_id=trace_id,
        )

    def delete_recall(self, recall_id: str, *, tenant_id: str, user_id: str) -> GovernanceActionResponse:
        trace_id = f"trace:{uuid4().hex}"
        ok = self.repository.delete_recall(recall_id, trace_id, tenant_id=tenant_id, user_id=user_id)
        return GovernanceActionResponse(
            ok=ok,
            action="delete",
            target_id=recall_id,
            trace_id=trace_id,
        )

    def get_audit(self, trace_id: str):
        return self.repository.get_audit_trace(trace_id)


memory_service = MemoryService()
