from __future__ import annotations

import json
from datetime import datetime, timezone
from typing import Any
from uuid import uuid4

import httpx
from httpx import HTTPStatusError

from app.schemas.common import AuditItem, FactItem, RecallItem, RelationItem
from app.schemas.events import EventAppendRequest, EventAppendResponse


class SurrealRepository:
    def __init__(
        self,
        *,
        base_url: str,
        namespace: str,
        database: str,
        username: str | None = None,
        password: str | None = None,
        token: str | None = None,
        timeout_seconds: float = 5.0,
    ) -> None:
        self.base_url = base_url.rstrip("/")
        self.namespace = namespace
        self.database = database
        self.client = httpx.Client(timeout=timeout_seconds)
        headers = {
            "Accept": "application/json",
            "Content-Type": "text/plain",
            "NS": namespace,
            "DB": database,
        }
        if token:
            headers["Authorization"] = f"Bearer {token}"
        self.client.headers.update(headers)
        if username and password and not token:
            self._signin(username=username, password=password, namespace=namespace, database=database)

    def _signin(self, *, username: str, password: str, namespace: str, database: str) -> None:
        payload_candidates = [
            {"user": username, "pass": password, "ns": namespace, "db": database},
            {"user": username, "pass": password},
        ]
        for payload in payload_candidates:
            response = self.client.post(
                f"{self.base_url}/signin",
                json=payload,
                headers={"Accept": "application/json"},
            )
            if response.status_code >= 400:
                continue
            data = response.json()
            token = None
            if isinstance(data, str):
                token = data
            elif isinstance(data, dict):
                token = data.get("token")
            if token:
                self.client.headers["Authorization"] = f"Bearer {token}"
                return
        # Fallback for setups that still allow basic auth on /sql.
        self.client.auth = (username, password)

    def _sql(self, query: str) -> list[dict[str, Any]]:
        query_with_context = f"USE NS {self.namespace} DB {self.database};{query}"
        response = self.client.post(f"{self.base_url}/sql", content=query_with_context)
        try:
            response.raise_for_status()
        except HTTPStatusError as exc:
            raise RuntimeError(f"surrealdb http error: {response.status_code} body={response.text}") from exc
        payload = response.json()
        if isinstance(payload, dict):
            payload = [payload]
        if not isinstance(payload, list):
            return []
        if payload and payload[0].get("status") == "OK" and payload[0].get("result") is None:
            payload = payload[1:]
        for item in payload:
            status = item.get("status")
            if status and status != "OK":
                detail = item.get("detail") or item.get("result") or "unknown surrealdb sql error"
                raise RuntimeError(str(detail))
        return payload

    @staticmethod
    def _q(value: Any) -> str:
        return json.dumps(value, ensure_ascii=False)

    @staticmethod
    def _rows(stmt: dict[str, Any] | None) -> list[dict[str, Any]]:
        if not stmt:
            return []
        rows = stmt.get("result")
        if rows is None:
            return []
        if isinstance(rows, list):
            return rows
        return []

    @staticmethod
    def _to_fact_item(row: dict[str, Any]) -> FactItem:
        return FactItem(
            id=str(row.get("id", "")),
            fact_key=row.get("fact_key", ""),
            fact_value=row.get("fact_value"),
            scope=row.get("scope"),
            version=int(row.get("version", 1)),
            status=row.get("status", "active"),
            source_trace=row.get("source_trace", ""),
        )

    @staticmethod
    def _to_recall_item(row: dict[str, Any]) -> RecallItem:
        return RecallItem(
            id=str(row.get("id", "")),
            text=row.get("text", ""),
            score=float(row.get("score", 0.0)),
            event_ref=str(row.get("event_ref", "")),
        )

    @staticmethod
    def _to_relation_item(row: dict[str, Any]) -> RelationItem:
        from_value = row.get("from_ref")
        to_value = row.get("to_ref")
        return RelationItem(
            id=str(row.get("id", "")),
            **{
                "from": str(from_value if from_value is not None else ""),
                "to": str(to_value if to_value is not None else ""),
                "rel_type": row.get("rel_type", ""),
                "weight": float(row.get("weight", 1.0)),
                "scope": row.get("scope"),
            },
        )

    def append_event(
        self,
        payload: EventAppendRequest,
        *,
        tenant_id: str,
        user_id: str,
        agent_id: str,
    ) -> EventAppendResponse:
        event_id = f"event:{uuid4().hex}"
        trace_id = f"trace:{uuid4().hex}"
        conversation_id = payload.conversation_id or f"conv:{uuid4().hex}"
        ts_expr = (
            f"type::datetime({self._q(payload.ts.isoformat())})"
            if payload.ts is not None
            else "time::now()"
        )
        query = f"""
CREATE type::thing('event_log', {self._q(event_id)}) SET
  tenant_id = {self._q(tenant_id)},
  user_id = {self._q(user_id)},
  agent_id = {self._q(payload.agent or agent_id)},
  session_id = {self._q(conversation_id)},
  content = {self._q(payload.content)},
  event_type = {self._q(payload.event_type.value)},
  source = 'api',
  ts = {ts_expr},
  meta = {self._q(payload.meta or {})};
CREATE audit_log SET
  trace_id = {self._q(trace_id)},
  tenant_id = {self._q(tenant_id)},
  user_id = {self._q(user_id)},
  action = 'propose',
  target_type = 'event',
  target_id = {self._q(event_id)},
  reason = 'event accepted',
  operator = 'agent',
  ts = time::now();
"""
        self._sql(query)
        return EventAppendResponse(event_id=event_id, trace_id=trace_id, conversation_id=conversation_id)

    def list_facts(self, scope: str | None, *, tenant_id: str, user_id: str) -> list[FactItem]:
        scope_filter = "" if scope is None else f" AND scope = {self._q(scope)}"
        query = (
            "SELECT id,fact_key,fact_value,scope,version,status,source_trace,updated_at "
            f"FROM fact_truth WHERE tenant_id = {self._q(tenant_id)} AND user_id = {self._q(user_id)} "
            "AND status = 'active'"
            f"{scope_filter} ORDER BY updated_at DESC LIMIT 200;"
        )
        data = self._sql(query)
        rows = self._rows(data[0] if data else None)
        return [self._to_fact_item(x) for x in rows]

    def query_memory(
        self,
        query_text: str,
        top_k: int,
        *,
        tenant_id: str,
        user_id: str,
    ) -> tuple[list[FactItem], list[RecallItem], list[RelationItem]]:
        facts_q = (
            "SELECT id,fact_key,fact_value,scope,version,status,source_trace,updated_at "
            f"FROM fact_truth WHERE tenant_id = {self._q(tenant_id)} AND user_id = {self._q(user_id)} "
            "AND status = 'active' ORDER BY updated_at DESC "
            f"LIMIT {int(top_k)};"
        )
        # V1 先使用文本匹配；向量检索可在 embedding 写入后替换为 vector::similarity 查询。
        recalls_q = (
            "SELECT id,text,event_ref,decay_score AS score,created_at "
            f"FROM recall_chunk WHERE tenant_id = {self._q(tenant_id)} AND user_id = {self._q(user_id)} "
            f"AND string::contains(string::lowercase(text), string::lowercase({self._q(query_text)})) "
            "ORDER BY created_at DESC "
            f"LIMIT {int(top_k)};"
        )
        relations_q = (
            "SELECT id,in AS from_ref,out AS to_ref,rel_type,weight,scope "
            f"FROM relation WHERE tenant_id = {self._q(tenant_id)} AND user_id = {self._q(user_id)} "
            f"LIMIT {int(top_k)};"
        )
        raw = self._sql(facts_q + recalls_q + relations_q)
        facts_rows = self._rows(raw[0] if len(raw) > 0 else None)
        recalls_rows = self._rows(raw[1] if len(raw) > 1 else None)
        relations_rows = self._rows(raw[2] if len(raw) > 2 else None)
        return (
            [self._to_fact_item(x) for x in facts_rows],
            [self._to_recall_item(x) for x in recalls_rows],
            [self._to_relation_item(x) for x in relations_rows],
        )

    def override_fact(
        self,
        fact_id: str,
        fact_value: Any,
        scope: str | None,
        trace_id: str,
        *,
        tenant_id: str,
        user_id: str,
    ) -> bool:
        fact_ref = f"type::thing('fact_truth', {self._q(fact_id)})"
        scope_clause = "" if scope is None else f", scope = {self._q(scope)}"
        audit_doc = {
            "trace_id": trace_id,
            "tenant_id": tenant_id,
            "user_id": user_id,
            "action": "override",
            "target_type": "fact",
            "target_id": fact_id,
            "reason": "fact overridden",
            "operator": "user",
        }
        query = (
            f"UPDATE {fact_ref} SET fact_value = {self._q(fact_value)}{scope_clause}, "
            "version += 1, status = 'active', updated_at = time::now();"
            "CREATE audit_log SET "
            f"trace_id = {self._q(audit_doc['trace_id'])},"
            f"tenant_id = {self._q(audit_doc['tenant_id'])},"
            f"user_id = {self._q(audit_doc['user_id'])},"
            "action = 'override',"
            "target_type = 'fact',"
            f"target_id = {self._q(audit_doc['target_id'])},"
            "reason = 'fact overridden',"
            "operator = 'user',"
            "ts = time::now();"
        )
        try:
            self._sql(query)
            return True
        except Exception:
            return False

    def revoke_fact(
        self,
        fact_id: str,
        trace_id: str,
        reason: str | None,
        *,
        tenant_id: str,
        user_id: str,
    ) -> bool:
        fact_ref = f"type::thing('fact_truth', {self._q(fact_id)})"
        audit_doc = {
            "trace_id": trace_id,
            "tenant_id": tenant_id,
            "user_id": user_id,
            "action": "revoke",
            "target_type": "fact",
            "target_id": fact_id,
            "reason": reason or "fact revoked",
            "operator": "user",
        }
        query = (
            f"UPDATE {fact_ref} SET status = 'revoked', updated_at = time::now();"
            "CREATE audit_log SET "
            f"trace_id = {self._q(audit_doc['trace_id'])},"
            f"tenant_id = {self._q(audit_doc['tenant_id'])},"
            f"user_id = {self._q(audit_doc['user_id'])},"
            "action = 'revoke',"
            "target_type = 'fact',"
            f"target_id = {self._q(audit_doc['target_id'])},"
            f"reason = {self._q(audit_doc['reason'])},"
            "operator = 'user',"
            "ts = time::now();"
        )
        try:
            self._sql(query)
            return True
        except Exception:
            return False

    def delete_recall(self, recall_id: str, trace_id: str, *, tenant_id: str, user_id: str) -> bool:
        recall_ref = f"type::thing('recall_chunk', {self._q(recall_id)})"
        audit_doc = {
            "trace_id": trace_id,
            "tenant_id": tenant_id,
            "user_id": user_id,
            "action": "delete",
            "target_type": "recall",
            "target_id": recall_id,
            "reason": "recall deleted",
            "operator": "user",
        }
        query = (
            f"DELETE {recall_ref};"
            "CREATE audit_log SET "
            f"trace_id = {self._q(audit_doc['trace_id'])},"
            f"tenant_id = {self._q(audit_doc['tenant_id'])},"
            f"user_id = {self._q(audit_doc['user_id'])},"
            "action = 'delete',"
            "target_type = 'recall',"
            f"target_id = {self._q(audit_doc['target_id'])},"
            "reason = 'recall deleted',"
            "operator = 'user',"
            "ts = time::now();"
        )
        try:
            self._sql(query)
            return True
        except Exception:
            return False

    def get_audit_trace(self, trace_id: str) -> list[AuditItem]:
        query = (
            "SELECT action,target_type,target_id,reason,operator,ts "
            f"FROM audit_log WHERE trace_id = {self._q(trace_id)} ORDER BY ts ASC LIMIT 200;"
        )
        data = self._sql(query)
        rows = self._rows(data[0] if data else None)
        out: list[AuditItem] = []
        for row in rows:
            ts = row.get("ts")
            ts_dt = datetime.now(timezone.utc)
            if isinstance(ts, str):
                try:
                    ts_dt = datetime.fromisoformat(ts.replace("Z", "+00:00"))
                except ValueError:
                    pass
            out.append(
                AuditItem(
                    action=row.get("action", ""),
                    target_type=row.get("target_type", ""),
                    target_id=row.get("target_id", ""),
                    reason=row.get("reason"),
                    operator=row.get("operator", "system"),
                    ts=ts_dt,
                )
            )
        return out
