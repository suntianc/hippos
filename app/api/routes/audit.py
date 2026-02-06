from fastapi import APIRouter

from app.schemas.common import AuditTraceResponse
from app.services.memory_service import memory_service

router = APIRouter(prefix="/api/v1", tags=["Audit"])


@router.get("/memory.audit/{trace_id}", response_model=AuditTraceResponse)
def get_audit(trace_id: str) -> AuditTraceResponse:
    items = memory_service.get_audit(trace_id)
    return AuditTraceResponse(trace_id=trace_id, items=items)
