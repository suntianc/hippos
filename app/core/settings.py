from functools import lru_cache
import os


class Settings:
    app_name: str
    default_tenant_id: str
    default_user_id: str
    default_agent_id: str
    default_top_k: int
    default_hops: int
    memory_backend: str
    surreal_url: str
    surreal_namespace: str
    surreal_database: str
    surreal_username: str | None
    surreal_password: str | None
    surreal_token: str | None
    surreal_timeout_seconds: float

    def __init__(self) -> None:
        self.app_name = os.getenv("APP_NAME", "personal-memory-system")
        self.default_tenant_id = os.getenv("DEFAULT_TENANT_ID", "tenant:default")
        self.default_user_id = os.getenv("DEFAULT_USER_ID", "user:me")
        self.default_agent_id = os.getenv("DEFAULT_AGENT_ID", "agent:default")
        self.default_top_k = int(os.getenv("DEFAULT_TOP_K", "10"))
        self.default_hops = int(os.getenv("DEFAULT_HOPS", "1"))
        self.memory_backend = os.getenv("MEMORY_BACKEND", "inmemory")
        self.surreal_url = os.getenv("SURREAL_URL", "http://127.0.0.1:12470")
        self.surreal_namespace = os.getenv("SURREAL_NAMESPACE", "mem_ns")
        self.surreal_database = os.getenv("SURREAL_DATABASE", "mem_db")
        self.surreal_username = os.getenv("SURREAL_USERNAME")
        self.surreal_password = os.getenv("SURREAL_PASSWORD")
        self.surreal_token = os.getenv("SURREAL_TOKEN")
        self.surreal_timeout_seconds = float(os.getenv("SURREAL_TIMEOUT_SECONDS", "5"))


@lru_cache
def get_settings() -> Settings:
    return Settings()
