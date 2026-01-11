# Hippos API Reference

**High-Performance Context Management Service for AI Agents**

## Base URL

```
http://localhost:8080
```

## Authentication

All API endpoints require authentication. Two methods are supported:

### API Key Authentication

```bash
curl -H "Authorization: ApiKey YOUR_API_KEY" http://localhost:8080/api/v1/sessions
```

### JWT Bearer Token

```bash
curl -H "Authorization: Bearer YOUR_JWT_TOKEN" http://localhost:8080/api/v1/sessions
```

### Default Credentials (Development)

| Credential | Value |
|------------|-------|
| API Key | `dev-api-key` |
| JWT Secret | `dev-secret-change-in-production-min-32-chars` |

---

## Sessions API

### Create Session

Create a new conversation session.

**Endpoint:** `POST /api/v1/sessions`

**Request Body:**

```json
{
  "name": "session-name",
  "description": "Optional session description",
  "max_turns": 100,
  "summary_limit": 10,
  "semantic_search_enabled": true,
  "auto_summarize": false
}
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `name` | string | Yes | - | Session name |
| `description` | string | No | `""` | Session description |
| `max_turns` | integer | No | 100 | Maximum number of turns |
| `summary_limit` | integer | No | 10 | Turns between summaries |
| `semantic_search_enabled` | boolean | No | true | Enable semantic search |
| `auto_summarize` | boolean | No | false | Auto-generate summaries |

**Response (201 Created):**

```json
{
  "id": "session_abc123",
  "created_at": "2024-01-15T10:30:00Z"
}
```

**Example:**

```bash
curl -X POST http://localhost:8080/api/v1/sessions \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{"name": "my-session", "description": "Test session"}'
```

---

### List Sessions

Retrieve a paginated list of sessions.

**Endpoint:** `GET /api/v1/sessions`

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | integer | 1 | Page number (1-based) |
| `page_size` | integer | 20 | Items per page (max 100) |
| `status` | string | "all" | Filter: "active", "archived", "all" |

**Response (200 OK):**

```json
{
  "sessions": [
    {
      "id": "session_abc123",
      "tenant_id": "tenant_1",
      "name": "my-session",
      "description": "Session description",
      "created_at": "2024-01-15T10:30:00Z",
      "last_active_at": "2024-01-15T11:00:00Z",
      "status": "active",
      "config": {
        "summary_limit": 10,
        "max_turns": 100,
        "semantic_search_enabled": true,
        "auto_summarize": false
      },
      "stats": {
        "total_turns": 5,
        "total_tokens": 1500,
        "storage_size": 8192,
        "last_indexed_at": "2024-01-15T11:00:00Z"
      }
    }
  ],
  "total": 1,
  "page": 1,
  "page_size": 20
}
```

**Example:**

```bash
curl "http://localhost:8080/api/v1/sessions?page=1&page_size=20&status=active" \
  -H "Authorization: ApiKey dev-api-key"
```

---

### Get Session

Retrieve a specific session by ID.

**Endpoint:** `GET /api/v1/sessions/{id}`

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string | Session unique identifier |

**Response (200 OK):**

```json
{
  "id": "session_abc123",
  "tenant_id": "tenant_1",
  "name": "my-session",
  "description": "Session description",
  "created_at": "2024-01-15T10:30:00Z",
  "last_active_at": "2024-01-15T11:00:00Z",
  "status": "active",
  "config": {
    "summary_limit": 10,
    "max_turns": 100,
    "semantic_search_enabled": true,
    "auto_summarize": false
  },
  "stats": {
    "total_turns": 5,
    "total_tokens": 1500,
    "storage_size": 8192,
    "last_indexed_at": "2024-01-15T11:00:00Z"
  }
}
```

**Example:**

```bash
curl http://localhost:8080/api/v1/sessions/session_abc123 \
  -H "Authorization: ApiKey dev-api-key"
```

---

### Update Session

Update session properties.

**Endpoint:** `PUT /api/v1/sessions/{id}`

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string | Session unique identifier |

**Request Body:**

```json
{
  "name": "updated-name",
  "description": "Updated description",
  "max_turns": 200,
  "status": "active"
}
```

**Response (200 OK):**

```json
{
  "id": "session_abc123",
  "message": "Session updated successfully"
}
```

**Example:**

```bash
curl -X PUT http://localhost:8080/api/v1/sessions/session_abc123 \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{"name": "updated-name"}'
```

---

### Delete Session

Delete a session and all its turns.

**Endpoint:** `DELETE /api/v1/sessions/{id}`

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string | Session unique identifier |

**Response (200 OK):**

```json
{
  "id": "session_abc123",
  "message": "Session deleted successfully"
}
```

**Example:**

```bash
curl -X DELETE http://localhost:8080/api/v1/sessions/session_abc123 \
  -H "Authorization: ApiKey dev-api-key"
```

---

## Turns API

### Add Turn

Add a new turn (message) to a session.

**Endpoint:** `POST /api/v1/sessions/{session_id}/turns`

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `session_id` | string | Session unique identifier |

**Request Body:**

```json
{
  "role": "user",
  "content": "User message content",
  "metadata": {
    "custom_key": "custom_value"
  }
}
```

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `role` | string | Yes | "user" or "assistant" |
| `content` | string | Yes | Message content |
| `metadata` | object | No | Custom metadata |

**Response (201 Created):**

```json
{
  "id": "turn_xyz789",
  "session_id": "session_abc123",
  "turn_number": 1,
  "created_at": "2024-01-15T11:00:00Z",
  "message_count": 1,
  "token_count": 50
}
```

**Example:**

```bash
curl -X POST http://localhost:8080/api/v1/sessions/session_abc123/turns \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{"role": "user", "content": "Hello, I need help with Rust programming"}'
```

---

### List Turns

List all turns in a session with pagination.

**Endpoint:** `GET /api/v1/sessions/{session_id}/turns`

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `session_id` | string | Session unique identifier |

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | integer | 1 | Page number |
| `page_size` | integer | 50 | Items per page |

**Response (200 OK):**

```json
{
  "turns": [
    {
      "id": "turn_xyz789",
      "session_id": "session_abc123",
      "turn_number": 1,
      "created_at": "2024-01-15T11:00:00Z",
      "role": "user",
      "content": "User message content",
      "metadata": {}
    }
  ],
  "total": 1,
  "page": 1,
  "page_size": 50
}
```

**Example:**

```bash
curl "http://localhost:8080/api/v1/sessions/session_abc123/turns?page=1&page_size=50" \
  -H "Authorization: ApiKey dev-api-key"
```

---

### Get Turn

Retrieve a specific turn by ID.

**Endpoint:** `GET /api/v1/sessions/{session_id}/turns/{turn_id}`

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `session_id` | string | Session unique identifier |
| `turn_id` | string | Turn unique identifier |

**Response (200 OK):**

```json
{
  "id": "turn_xyz789",
  "session_id": "session_abc123",
  "turn_number": 1,
  "created_at": "2024-01-15T11:00:00Z",
  "role": "user",
  "content": "User message content",
  "metadata": {},
  "token_count": 50
}
```

**Example:**

```bash
curl http://localhost:8080/api/v1/sessions/session_abc123/turns/turn_xyz789 \
  -H "Authorization: ApiKey dev-api-key"
```

---

### Delete Turn

Delete a specific turn.

**Endpoint:** `DELETE /api/v1/sessions/{session_id}/turns/{turn_id}`

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `session_id` | string | Session unique identifier |
| `turn_id` | string | Turn unique identifier |

**Response (200 OK):**

```json
{
  "id": "turn_xyz789",
  "message": "Turn deleted successfully"
}
```

**Example:**

```bash
curl -X DELETE http://localhost:8080/api/v1/sessions/session_abc123/turns/turn_xyz789 \
  -H "Authorization: ApiKey dev-api-key"
```

---

## Search API

### Hybrid Search

Combine semantic and keyword search within a session.

**Endpoint:** `GET /api/v1/sessions/{session_id}/search`

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `session_id` | string | Session unique identifier |

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `q` | string | - | Search query (required) |
| `limit` | integer | 10 | Maximum results |
| `strategy` | string | "hybrid" | "semantic", "fulltext", "hybrid" |

**Response (200 OK):**

```json
{
  "results": [
    {
      "id": "turn_xyz789",
      "score": 0.95,
      "type": "semantic",
      "content": "Matching content...",
      "metadata": {
        "turn_number": 1,
        "created_at": "2024-01-15T11:00:00Z"
      }
    }
  ],
  "total_results": 1,
  "search_time_ms": 15
}
```

**Example:**

```bash
curl "http://localhost:8080/api/v1/sessions/session_abc123/search?q=rust+programming&limit=10" \
  -H "Authorization: ApiKey dev-api-key"
```

---

### Semantic Search Only

Pure vector-based semantic search with optional similarity threshold.

**Endpoint:** `POST /api/v1/sessions/{session_id}/search/semantic`

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `session_id` | string | Session unique identifier |

**Request Body:**

```json
{
  "query": "What was discussed about Rust programming?",
  "limit": 10,
  "threshold": 0.7
}
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | string | Yes | - | Semantic search query |
| `limit` | integer | No | 10 | Maximum results |
| `threshold` | number | No | 0.0 | Similarity threshold (0-1) |

**Response (200 OK):**

```json
{
  "results": [
    {
      "id": "turn_xyz789",
      "score": 0.89,
      "content": "Rust programming discussion...",
      "metadata": {
        "turn_number": 5,
        "created_at": "2024-01-15T12:00:00Z"
      }
    }
  ],
  "total_results": 1,
  "search_time_ms": 25
}
```

**Example:**

```bash
curl -X POST http://localhost:8080/api/v1/sessions/session_abc123/search/semantic \
  -H "Authorization: ApiKey dev-api-key" \
  -H "Content-Type: application/json" \
  -d '{"query": "Rust ownership model", "limit": 5}'
```

---

### Get Recent Context

Retrieve the most recent turns for context.

**Endpoint:** `GET /api/v1/sessions/{session_id}/context/recent`

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `session_id` | string | Session unique identifier |

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit` | integer | 10 | Number of recent turns |

**Response (200 OK):**

```json
{
  "turns": [
    {
      "id": "turn_abc123",
      "turn_number": 10,
      "role": "assistant",
      "content": "Based on our discussion...",
      "created_at": "2024-01-15T14:00:00Z"
    }
  ],
  "total": 1
}
```

**Example:**

```bash
curl "http://localhost:8080/api/v1/sessions/session_abc123/context/recent?limit=5" \
  -H "Authorization: ApiKey dev-api-key"
```

---

## Health & Metrics API

### Full Health Check

Returns comprehensive health status including all subsystem checks.

**Endpoint:** `GET /health`

**Response (200 OK):**

```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T12:00:00Z",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "checks": [
    {
      "name": "database",
      "status": "healthy",
      "message": "Connected",
      "latency_ms": 5
    }
  ]
}
```

**Example:**

```bash
curl http://localhost:8080/health
```

---

### Liveness Check

Simple liveness probe. Always returns "OK" if the service is running.

**Endpoint:** `GET /health/live`

**Response:** `OK` (200 OK)

**Example:**

```bash
curl http://localhost:8080/health/live
```

---

### Readiness Check

Readiness probe that checks if the service is ready to accept traffic.

**Endpoint:** `GET /health/ready`

**Response:**
- `Ready` (200 OK) - Service is ready
- `Not Ready` (503 Service Unavailable) - Service is not ready

**Example:**

```bash
curl http://localhost:8080/health/ready
```

---

### Prometheus Metrics

Returns Prometheus-format metrics.

**Endpoint:** `GET /metrics`

**Response (200 OK):**

```
# HELP http_requests_total Total HTTP requests
# TYPE http_requests_total counter
http_requests_total 1234
# HELP http_request_duration_seconds HTTP request duration in seconds
# TYPE http_request_duration_seconds histogram
http_request_duration_seconds_sum 123.456
http_request_duration_seconds_count 1234
# HELP active_connections Active HTTP connections
# TYPE active_connections gauge
active_connections 5
# HELP sessions_active Active sessions
# TYPE sessions_active gauge
sessions_active 10
# HELP turns_total Total turns stored
# TYPE turns_total counter
turns_total 150
# HELP search_requests_total Total search requests
# TYPE search_requests_total counter
search_requests_total 500
# HELP errors_total Total errors
# TYPE errors_total counter
errors_total 5
```

**Example:**

```bash
curl http://localhost:8080/metrics
```

---

### Version Information

Returns version and uptime information.

**Endpoint:** `GET /version`

**Response (200 OK):**

```json
{
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "timestamp": "2024-01-15T12:00:00Z"
}
```

**Example:**

```bash
curl http://localhost:8080/version
```

---

## Error Responses

All errors return a consistent error format:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Request parameter validation failed",
    "details": {
      "field": "name",
      "reason": "Name cannot be empty"
    }
  },
  "request_id": "req_abc123",
  "timestamp": "2024-01-15T12:00:00Z"
}
```

### Common Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `UNAUTHORIZED` | 401 | Missing or invalid authentication token |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `NOT_FOUND` | 404 | Requested resource does not exist |
| `VALIDATION_ERROR` | 400 | Request parameter validation failed |
| `RATE_LIMITED` | 429 | Request rate limit exceeded |
| `INTERNAL_ERROR` | 500 | Server internal error |

---

## Rate Limiting

Hippos implements Token Bucket rate limiting:

| Limit Type | Default | Description |
|------------|---------|-------------|
| Global | 1000/min | Requests per minute globally |
| Per-Session | 100/min | Requests per minute per session |
| Per-Endpoint | Configurable | Custom limits per endpoint |

**Response Headers:**

| Header | Description |
|--------|-------------|
| `X-RateLimit-Limit` | Allowed requests |
| `X-RateLimit-Remaining` | Remaining requests |
| `X-RateLimit-Reset` | Reset timestamp |
| `Retry-After` | Suggested wait time (seconds) |

---

## Quick Reference

### Endpoints Summary

| Category | Method | Endpoint | Description |
|----------|--------|----------|-------------|
| **Sessions** | POST | `/api/v1/sessions` | Create session |
| | GET | `/api/v1/sessions` | List sessions |
| | GET | `/api/v1/sessions/{id}` | Get session |
| | PUT | `/api/v1/sessions/{id}` | Update session |
| | DELETE | `/api/v1/sessions/{id}` | Delete session |
| **Turns** | POST | `/api/v1/sessions/{id}/turns` | Add turn |
| | GET | `/api/v1/sessions/{id}/turns` | List turns |
| | GET | `/api/v1/sessions/{id}/turns/{turn_id}` | Get turn |
| | DELETE | `/api/v1/sessions/{id}/turns/{turn_id}` | Delete turn |
| **Search** | GET | `/api/v1/sessions/{id}/search` | Hybrid search |
| | POST | `/api/v1/sessions/{id}/search/semantic` | Semantic search |
| | GET | `/api/v1/sessions/{id}/context/recent` | Recent context |
| **Health** | GET | `/health` | Full health check |
| | GET | `/health/live` | Liveness probe |
| | GET | `/health/ready` | Readiness probe |
| | GET | `/metrics` | Prometheus metrics |
| | GET | `/version` | Version info |

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `EXOCORTEX_SERVER_PORT` | 8080 | Server port |
| `EXOCORTEX_DATABASE_URL` | ws://localhost:8000 | SurrealDB URL |
| `EXOCORTEX_API_KEY` | dev-api-key | API key |
| `EXOCORTEX_LOG_LEVEL` | info | Log level |

---

## SDK & Clients

### Official Clients

| Language | Repository |
|----------|------------|
| TypeScript/JavaScript | (Coming soon) |
| Python | (Coming soon) |

### Community Clients

| Language | Repository |
|----------|------------|
| Rust | Built-in |

---

*Last Updated: 2024-01-11*
