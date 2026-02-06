from dataclasses import dataclass
from fastapi import Request

from app.core.settings import get_settings


@dataclass
class IdentityContext:
    tenant_id: str
    user_id: str
    agent_id: str


def resolve_identity(request: Request) -> IdentityContext:
    settings = get_settings()
    return IdentityContext(
        tenant_id=request.headers.get("x-tenant-id", settings.default_tenant_id),
        user_id=request.headers.get("x-user-id", settings.default_user_id),
        agent_id=request.headers.get("x-agent-id", settings.default_agent_id),
    )
