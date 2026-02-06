from fastapi import APIRouter, Request

from app.core.identity import resolve_identity
from app.schemas.events import (
    EventAppendRequest,
    EventAppendResponse,
    EventBatchAppendRequest,
    EventBatchAppendResponse,
)
from app.services.memory_service import memory_service

router = APIRouter(prefix="/api/v1", tags=["Events"])


@router.post("/events.append", response_model=EventAppendResponse)
def append_event(payload: EventAppendRequest, request: Request) -> EventAppendResponse:
    identity = resolve_identity(request)
    return memory_service.append_event(
        payload,
        tenant_id=identity.tenant_id,
        user_id=identity.user_id,
        agent_id=identity.agent_id,
    )


@router.post("/events.batch_append", response_model=EventBatchAppendResponse)
def batch_append_events(payload: EventBatchAppendRequest, request: Request) -> EventBatchAppendResponse:
    identity = resolve_identity(request)
    items = [
        memory_service.append_event(
            item,
            tenant_id=identity.tenant_id,
            user_id=identity.user_id,
            agent_id=identity.agent_id,
        )
        for item in payload.events
    ]
    return EventBatchAppendResponse(items=items)
