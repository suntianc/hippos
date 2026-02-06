from fastapi import APIRouter, Query, Request

from app.core.identity import resolve_identity
from app.schemas.common import FactListResponse
from app.schemas.query import MemoryQueryRequest, MemoryQueryResponse
from app.services.memory_service import memory_service

router = APIRouter(prefix="/api/v1", tags=["Query"])


@router.post("/memory.query", response_model=MemoryQueryResponse)
def query_memory(payload: MemoryQueryRequest, request: Request) -> MemoryQueryResponse:
    identity = resolve_identity(request)
    return memory_service.query_memory(payload, tenant_id=identity.tenant_id, user_id=identity.user_id)


@router.get("/memory.facts", response_model=FactListResponse)
def list_facts(request: Request, scope: str | None = Query(default=None)) -> FactListResponse:
    identity = resolve_identity(request)
    return FactListResponse(
        facts=memory_service.list_facts(scope, tenant_id=identity.tenant_id, user_id=identity.user_id)
    )
