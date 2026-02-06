from fastapi.testclient import TestClient

from app.main import app


client = TestClient(app)


def test_healthz() -> None:
    response = client.get("/healthz")
    assert response.status_code == 200
    assert response.json()["status"] == "ok"


def test_append_and_query() -> None:
    append_resp = client.post("/api/v1/events.append", json={"content": "记住我喜欢晨跑"})
    assert append_resp.status_code == 200
    body = append_resp.json()
    assert body["accepted"] is True
    assert body["conversation_id"]

    query_resp = client.post("/api/v1/memory.query", json={"query_text": "晨跑"})
    assert query_resp.status_code == 200
    payload = query_resp.json()
    assert "facts" in payload
    assert "recalls" in payload
    assert "relations" in payload
    assert "trace_id" in payload
