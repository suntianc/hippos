from fastapi import FastAPI

from app.api.routes.audit import router as audit_router
from app.api.routes.events import router as events_router
from app.api.routes.governance import router as governance_router
from app.api.routes.query import router as query_router
from app.core.settings import get_settings

settings = get_settings()
app = FastAPI(title=settings.app_name, version="0.1.0")


@app.get("/healthz")
def healthz() -> dict[str, str]:
    return {"status": "ok"}


app.include_router(events_router)
app.include_router(query_router)
app.include_router(governance_router)
app.include_router(audit_router)
