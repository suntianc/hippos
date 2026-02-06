from fastapi import APIRouter, Request
from pydantic import BaseModel

from app.core.identity import resolve_identity
from app.schemas.common import FactOverrideRequest, GovernanceActionResponse
from app.services.memory_service import memory_service

router = APIRouter(prefix="/api/v1", tags=["Governance"])


class RevokeRequest(BaseModel):
    reason: str | None = None


@router.post("/memory.facts/{id}/override", response_model=GovernanceActionResponse)
def override_fact(id: str, payload: FactOverrideRequest, request: Request) -> GovernanceActionResponse:
    identity = resolve_identity(request)
    return memory_service.override_fact(id, payload, tenant_id=identity.tenant_id, user_id=identity.user_id)


@router.post("/memory.facts/{id}/revoke", response_model=GovernanceActionResponse)
def revoke_fact(id: str, request: Request, payload: RevokeRequest | None = None) -> GovernanceActionResponse:
    identity = resolve_identity(request)
    reason = payload.reason if payload else None
    return memory_service.revoke_fact(id, reason, tenant_id=identity.tenant_id, user_id=identity.user_id)


@router.delete("/memory.recalls/{id}", response_model=GovernanceActionResponse)
def delete_recall(id: str, request: Request) -> GovernanceActionResponse:
    identity = resolve_identity(request)
    return memory_service.delete_recall(id, tenant_id=identity.tenant_id, user_id=identity.user_id)
