from app.core.settings import get_settings
from app.repository.in_memory_repository import InMemoryRepository
from app.repository.surreal_repository import SurrealRepository
from app.repository.types import MemoryRepository


def create_repository() -> MemoryRepository:
    settings = get_settings()
    if settings.memory_backend == "surreal":
        return SurrealRepository(
            base_url=settings.surreal_url,
            namespace=settings.surreal_namespace,
            database=settings.surreal_database,
            username=settings.surreal_username,
            password=settings.surreal_password,
            token=settings.surreal_token,
            timeout_seconds=settings.surreal_timeout_seconds,
        )
    return InMemoryRepository()
