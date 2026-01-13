"""Hippos client for session and turn management."""

from typing import Optional
from dataclasses import dataclass
from datetime import datetime

import httpx


@dataclass
class HipposSession:
    """Hippos session."""

    id: str
    tenant_id: str
    name: str
    description: Optional[str]
    created_at: datetime
    last_active_at: datetime
    status: str


@dataclass
class HipposTurn:
    """Hippos turn."""

    id: str
    session_id: str
    turn_number: int
    role: str
    content: str
    created_at: datetime


class HipposClient:
    """Client for Hippos API."""

    def __init__(self, base_url: str, api_key: str):
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self.http_client = httpx.AsyncClient(timeout=30.0)

    async def close(self):
        """Close the HTTP client."""
        await self.http_client.aclose()

    def _headers(self) -> dict:
        return {"Authorization": f"ApiKey {self.api_key}"}

    async def create_session(
        self,
        name: str,
        description: Optional[str] = None,
        tenant_id: str = "default",
    ) -> HipposSession:
        """Create a new session."""
        response = await self.http_client.post(
            f"{self.base_url}/api/v1/sessions",
            headers=self._headers(),
            json={
                "name": name,
                "description": description,
                "tenant_id": tenant_id,
            },
        )
        response.raise_for_status()
        data = response.json()
        return HipposSession(
            id=data["id"],
            tenant_id=tenant_id,
            name=name,
            description=description,
            created_at=datetime.fromisoformat(data["created_at"].replace("Z", "+00:00")),
            last_active_at=datetime.now(),
            status="Active",
        )

    async def get_session(self, session_id: str) -> Optional[HipposSession]:
        """Get a session by ID."""
        response = await self.http_client.get(
            f"{self.base_url}/api/v1/sessions/{session_id}",
            headers=self._headers(),
        )
        if response.status_code == 404:
            return None
        response.raise_for_status()
        data = response.json()
        return HipposSession(
            id=data["id"],
            tenant_id=data["tenant_id"],
            name=data["name"],
            description=data.get("description"),
            created_at=datetime.fromisoformat(data["created_at"].replace("Z", "+00:00")),
            last_active_at=datetime.fromisoformat(data["last_active_at"].replace("Z", "+00:00")),
            status=data["status"],
        )

    async def list_sessions(
        self,
        page: int = 1,
        page_size: int = 20,
    ) -> tuple[list[HipposSession], int]:
        """List all sessions."""
        response = await self.http_client.get(
            f"{self.base_url}/api/v1/sessions",
            headers=self._headers(),
            params={"page": page, "page_size": page_size},
        )
        response.raise_for_status()
        data = response.json()
        sessions = []
        for s in data.get("sessions", []):
            sessions.append(
                HipposSession(
                    id=s["id"],
                    tenant_id=s["tenant_id"],
                    name=s["name"],
                    description=s.get("description"),
                    created_at=datetime.fromisoformat(s["created_at"].replace("Z", "+00:00")),
                    last_active_at=datetime.fromisoformat(
                        s["last_active_at"].replace("Z", "+00:00")
                    ),
                    status=s["status"],
                )
            )
        return sessions, data.get("total", 0)

    async def add_turn(
        self,
        session_id: str,
        role: str,
        content: str,
    ) -> HipposTurn:
        """Add a turn to a session."""
        response = await self.http_client.post(
            f"{self.base_url}/api/v1/sessions/{session_id}/turns",
            headers=self._headers(),
            json={"role": role, "content": content},
        )
        response.raise_for_status()
        data = response.json()
        return HipposTurn(
            id=data["id"],
            session_id=session_id,
            turn_number=data["turn_number"],
            role=role,
            content=content,
            created_at=datetime.fromisoformat(data["created_at"].replace("Z", "+00:00")),
        )

    async def list_turns(
        self,
        session_id: str,
        page: int = 1,
        page_size: int = 50,
    ) -> tuple[list[HipposTurn], int]:
        """List turns for a session."""
        response = await self.http_client.get(
            f"{self.base_url}/api/v1/sessions/{session_id}/turns",
            headers=self._headers(),
            params={"page": page, "page_size": page_size},
        )
        response.raise_for_status()
        data = response.json()
        turns = []
        for t in data.get("turns", []):
            turns.append(
                HipposTurn(
                    id=t["id"],
                    session_id=session_id,
                    turn_number=t["turn_number"],
                    role=t["role"],
                    content=t["content"],
                    created_at=datetime.fromisoformat(t["created_at"].replace("Z", "+00:00")),
                )
            )
        return turns, data.get("total", 0)

    async def search(
        self,
        session_id: str,
        query: str,
        limit: int = 5,
    ) -> list[dict]:
        """Search within a session."""
        response = await self.http_client.get(
            f"{self.base_url}/api/v1/sessions/{session_id}/search",
            headers=self._headers(),
            params={"q": query, "limit": limit},
        )
        response.raise_for_status()
        data = response.json()
        return data.get("results", [])

    async def health_check(self) -> bool:
        """Check if Hippos is healthy."""
        try:
            response = await self.http_client.get(
                f"{self.base_url}/health",
            )
            return response.status_code == 200
        except Exception:
            return False
