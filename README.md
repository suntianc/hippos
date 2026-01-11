# Hippos

**High-Performance Context Management Service for AI Agents**

Hippos is a Rust-based context management service designed to provide persistent conversation memory capabilities for Large Language Models (LLMs). It solves the context window limitation problem in long conversation scenarios by efficiently managing, indexing, and retrieving conversation context.

## 🚀 Quick Start

### Prerequisites

- **Rust**: 1.70.0 or later (2024 Edition)
- **SurrealDB**: 1.5.6 or later (optional, in-memory mode available)
- **Cargo**: Latest stable version

### Installation

```bash
# Clone the repository
git clone https://github.com/hippos/hippos.git
cd hippos

# Build the project
cargo build --release

# Run tests
cargo test --lib
```

### Running the Server

```bash
# Run with default configuration (in-memory database)
cargo run

# Run with custom configuration
EXOCORTEX_SERVER_PORT=8080 cargo run

# Run with custom config file
EXOCORTEX_CONFIG=/path/to/config.yaml cargo run
```

### Example API Calls

```bash
# Create a new session
curl -X POST http://localhost:8080/api/v1/sessions \
  -H "Content-Type: application/json" \
  -H "Authorization: ApiKey dev-api-key" \
  -d '{"name": "my-first-session", "description": "Test session"}'

# List all sessions
curl http://localhost:8080/api/v1/sessions \
  -H "Authorization: ApiKey dev-api-key"

# Add a turn to a session
curl -X POST http://localhost:8080/api/v1/sessions/{session_id}/turns \
  -H "Content-Type: application/json" \
  -H "Authorization: ApiKey dev-api-key" \
  -d '{"role": "user", "content": "Hello, I need help with Rust programming"}'

# Search within a session
curl "http://localhost:8080/api/v1/sessions/{session_id}/search?q=rust+programming" \
  -H "Authorization: ApiKey dev-api-key"

# Check health status
curl http://localhost:8080/health
```

## ✨ Features

### Context Management

- **Session Lifecycle**: Create, update, archive, and delete conversation sessions
- **Turn Management**: Store and retrieve individual conversation turns with metadata
- **Session Statistics**: Track token usage, turn counts, and storage metrics
- **Tenant Isolation**: Multi-tenant support with proper data isolation

### Hybrid Search Engine

- **Semantic Search**: Vector-based similarity search using transformer embeddings
- **Full-Text Search**: Keyword-based search with ranking
- **RRF Fusion**: Reciprocal Rank Fusion for improved hybrid search results
- **Real-time Indexing**: Automatic indexing of new content

### Content Processing

- **Dehydration**: Generate concise summaries of long conversations
- **Keyword Extraction**: Extract important topics and keywords
- **Context Compression**: Optimize context for LLM prompts
- **Batch Processing**: Efficient bulk operations for large datasets

### Security

- **API Key Authentication**: Simple yet effective API key authentication
- **JWT Token Validation**: Bearer token support with JSON Web Tokens
- **Role-Based Access Control (RBAC)**: Fine-grained permission management
- **Rate Limiting**: Token bucket algorithm for request throttling
- **Request Validation**: Input validation middleware

### Observability

- **Prometheus Metrics**: Comprehensive metrics endpoint
- **Health Checks**: Liveness, readiness, and full health checks
- **Structured Logging**: JSON-formatted logs with tracing
- **Request Tracking**: Track request latency and errors

## 🏗️ Architecture

### High-Level Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Hippos Service                        │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │   REST API  │  │  Security   │  │     Observability      │ │
│  │   (Axum)    │  │  Layer      │  │   (Prometheus/Logs)    │ │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘ │
│         │               │                     │                 │
│         └───────────────┼─────────────────────┘                 │
│                         ▼                                       │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                     Application State                      │ │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐  │ │
│  │  │Sessions  │ │  Turns   │ │ Retrieval│ │ Dehydration │  │ │
│  │  │Service   │ │ Service  │ │ Service  │ │ Service     │  │ │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────────┘  │ │
│  └───────────────────────────────────────────────────────────┘ │
│                         │                                       │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                    Storage Layer                           │ │
│  │  ┌──────────────────────────────────────────────────────┐  │ │
│  │  │              SurrealDB Connection Pool               │  │ │
│  │  └──────────────────────────────────────────────────────┘  │ │
│  └───────────────────────────────────────────────────────────┘ │
│                         │                                       │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                   Index Layer                              │ │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────────┐   │ │
│  │  │Vector Index  │ │Full-Text    │ │Embedding Model   │   │ │
│  │  │(DashMap)     │ │Index        │ │(Candle)          │   │ │
│  │  └──────────────┘ └──────────────┘ └──────────────────┘   │ │
│  └───────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility |
|-----------|---------------|
| **REST API (Axum)** | HTTP request handling, routing, request/response serialization |
| **Security Layer** | Authentication, authorization, rate limiting, validation |
| **Observability** | Metrics collection, health checks, logging, tracing |
| **Session Service** | Session CRUD operations, lifecycle management |
| **Turn Service** | Turn storage and retrieval, statistics tracking |
| **Retrieval Service** | Vector and hybrid search operations |
| **Dehydration Service** | Content summarization and context compression |
| **Storage Layer** | Database connection pooling, repository pattern |
| **Index Layer** | Vector embeddings, full-text search, in-memory indexing |

### Data Flow

1. **Incoming Request** → Security middleware validates authentication
2. **Validated Request** → Router dispatches to appropriate handler
3. **Handler** → Calls business logic services
4. **Services** → Interact with storage and index layers
5. **Response** → Formatted and returned to client
6. **Metrics** → Recorded throughout the request lifecycle

## 📚 API Documentation

### Base URL

```
http://localhost:8080
```

### Authentication

All API requests require authentication using one of the following methods:

```bash
# API Key authentication
curl -H "Authorization: ApiKey YOUR_API_KEY" ...

# JWT Bearer token authentication
curl -H "Authorization: Bearer YOUR_JWT_TOKEN" ...
```

**Default development credentials:**
- API Key: `dev-api-key`
- JWT Secret: `dev-secret-change-in-production-min-32-chars`

### Sessions API

#### Create Session

```http
POST /api/v1/sessions
Content-Type: application/json
```

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

**Response (201 Created):**

```json
{
  "id": "session_abc123",
  "created_at": "2024-01-15T10:30:00Z"
}
```

#### List Sessions

```http
GET /api/v1/sessions?page=1&page_size=20&status=active
```

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

#### Get Session

```http
GET /api/v1/sessions/{id}
```

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
  "config": { /* ... */ },
  "stats": { /* ... */ }
}
```

#### Update Session

```http
PUT /api/v1/sessions/{id}
Content-Type: application/json
```

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

#### Delete Session

```http
DELETE /api/v1/sessions/{id}
```

**Response (200 OK):**

```json
{
  "id": "session_abc123",
  "message": "Session deleted successfully"
}
```

### Turns API

#### Add Turn

```http
POST /api/v1/sessions/{id}/turns
Content-Type: application/json
```

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

#### List Turns

```http
GET /api/v1/sessions/{id}/turns?page=1&page_size=50
```

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

#### Get Turn

```http
GET /api/v1/sessions/{id}/turns/{turn_id}
```

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

#### Delete Turn

```http
DELETE /api/v1/sessions/{id}/turns/{turn_id}
```

**Response (200 OK):**

```json
{
  "id": "turn_xyz789",
  "message": "Turn deleted successfully"
}
```

### Search API

#### Hybrid Search

```http
GET /api/v1/sessions/{id}/search?q=search+query&limit=10&strategy=hybrid
```

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

#### Semantic Search Only

```http
POST /api/v1/sessions/{id}/search/semantic
Content-Type: application/json
```

**Request Body:**

```json
{
  "query": "What was discussed about Rust programming?",
  "limit": 10,
  "threshold": 0.7
}
```

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

### Health & Metrics API

#### Full Health Check

```http
GET /health
```

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

#### Liveness Check

```http
GET /health/live
```

**Response:** `OK` (200 OK)

#### Readiness Check

```http
GET /health/ready
```

**Response:** `Ready` (200 OK) or `Not Ready` (503 Service Unavailable)

#### Prometheus Metrics

```http
GET /metrics
```

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
...
```

#### Version Information

```http
GET /version
```

**Response (200 OK):**

```json
{
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "timestamp": "2024-01-15T12:00:00Z"
}
```

## ⚙️ Configuration

### Configuration File

Configuration is managed via `config.yaml` with environment variable override support.

```yaml
# Application Configuration
app:
  name: "hippos"
  environment: "development"

# Database Configuration
database:
  url: "ws://localhost:8000"
  namespace: "hippos"
  database: "sessions"
  username: "root"
  password: "root"
  min_connections: 5
  max_connections: 50
  connection_timeout: 30
  idle_timeout: 300

# Vector Database Configuration
vector:
  data_dir: "./data/lancedb"
  dimension: 384
  nlist: 1024
  nprobe: 32
  pq_m: 8
  distance_type: "cosine"

# Server Configuration
server:
  host: "0.0.0.0"
  port: 8080
  workers: 4
  request_timeout: 30
  max_request_size: 10485760

# Security Configuration
security:
  api_key: "dev-api-key-change-in-production"
  rate_limit_enabled: false
  global_rate_limit: 1000
  per_session_rate_limit: 100
  redis_url: "redis://localhost:6379"
  tls_enabled: false

# Logging Configuration
logging:
  level: "debug"
  structured: true
  log_dir: "./logs"
  file_max_size: 104857600
  file_max_count: 10

# Embedding Model Configuration
embedding:
  model_name: "all-MiniLM-L6-v2"
  model_path: null
  batch_size: 32
  use_gpu: false
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `EXOCORTEX_APP_NAME` | `hippos` | Application name |
| `EXOCORTEX_ENVIRONMENT` | `development` | Environment mode |
| `EXOCORTEX_DATABASE_URL` | `ws://localhost:8000` | SurrealDB connection URL |
| `EXOCORTEX_DATABASE_NAMESPACE` | `hippos` | Database namespace |
| `EXOCORTEX_DATABASE_NAME` | `sessions` | Database name |
| `EXOCORTEX_SERVER_HOST` | `0.0.0.0` | Server bind address |
| `EXOCORTEX_SERVER_PORT` | `8080` | Server port |
| `EXOCORTEX_SERVER_WORKERS` | `4` | Number of worker threads |
| `EXOCORTEX_API_KEY` | `dev-api-key` | Default API key |
| `EXOCORTEX_LOG_LEVEL` | `info` | Logging level |
| `EXOCORTEX_EMBEDDING_MODEL` | `all-MiniLM-L6-v2` | Embedding model name |

### Configuration Sections

#### Database Configuration

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `url` | String | `ws://localhost:8000` | SurrealDB WebSocket URL |
| `namespace` | String | `hippos` | Database namespace |
| `database` | String | `sessions` | Database name |
| `username` | String | `root` | Authentication username |
| `password` | String | `root` | Authentication password |
| `min_connections` | usize | `5` | Minimum connection pool size |
| `max_connections` | usize | `50` | Maximum connection pool size |
| `connection_timeout` | u64 | `30` | Connection timeout in seconds |
| `idle_timeout` | u64 | `300` | Idle connection timeout in seconds |

#### Vector Configuration

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `data_dir` | String | `./data/lancedb` | Vector database storage directory |
| `dimension` | usize | `384` | Embedding vector dimension |
| `nlist` | usize | `1024` | Number of IVF index lists |
| `nprobe` | usize | `32` | Number of probes for search |
| `distance_type` | String | `cosine` | Distance metric (cosine, euclidean, dot) |

#### Server Configuration

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `host` | String | `0.0.0.0` | Network interface to bind |
| `port` | u16 | `8080` | Server port |
| `workers` | usize | `4` | Tokio worker threads |
| `request_timeout` | u64 | `30` | Request timeout in seconds |
| `max_request_size` | usize | `10485760` | Max request body size (10MB) |

#### Security Configuration

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `api_key` | String | - | Default API key |
| `rate_limit_enabled` | bool | `false` | Enable rate limiting |
| `global_rate_limit` | u64 | `1000` | Global requests per minute |
| `per_session_rate_limit` | u64 | `100` | Per-session requests per minute |
| `redis_url` | String | `redis://localhost:6379` | Redis URL for rate limiting |
| `tls_enabled` | bool | `false` | Enable HTTPS/TLS |

#### Embedding Configuration

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `model_name` | String | `all-MiniLM-L6-v2` | HuggingFace model identifier |
| `model_path` | Option<String> | `None` | Local model path |
| `batch_size` | usize | `32` | Batch processing size |
| `use_gpu` | bool | `false` | Enable GPU acceleration |

## 📊 Metrics & Monitoring

### Available Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `http_requests_total` | Counter | Total HTTP requests received |
| `http_request_duration_seconds` | Histogram | Request duration in seconds |
| `active_connections` | Gauge | Current active connections |
| `sessions_active` | Gauge | Number of active sessions |
| `sessions_archived` | Gauge | Number of archived sessions |
| `turns_total` | Counter | Total turns stored |
| `search_requests_total` | Counter | Total search requests |
| `search_latency_seconds` | Histogram | Search request latency |
| `errors_total` | Counter | Total error count |

### Health Check Endpoints

| Endpoint | Description |
|----------|-------------|
| `/health` | Full health status with all checks |
| `/health/live` | Simple liveness check (always returns OK) |
| `/health/ready` | Readiness check (checks dependencies) |
| `/metrics` | Prometheus metrics endpoint |
| `/version` | Version and uptime information |

### Custom Health Checks

You can register custom health checks by implementing the `HealthCheck` trait:

```rust
use crate::observability::HealthCheckResult;

#[async_trait]
trait HealthCheck: Send + Sync {
    fn name(&self) -> String;
    async fn check(&self) -> HealthCheckResult;
}
```

## 🤖 MCP Server

Hippos can also run as a **Model Context Protocol (MCP)** server, allowing AI agents and applications to access its context management capabilities through a standardized protocol. This enables seamless integration with MCP-compatible clients like Claude Desktop, Cursor, and other AI tools.

### Running as MCP Server

```bash
# Build the release binary
cargo build --release

# Start Hippos in MCP mode (uses stdio for communication)
./target/release/hippos
```

Or set environment variable before running:

```bash
export HIPPOS_MCP_MODE=1
./target/release/hippos
```

When running in MCP mode, Hippos exposes two tools that clients can invoke:

When running in MCP mode, Hippos exposes two tools that clients can invoke:

### Available Tools

#### hippos_search

Perform hybrid search combining semantic and keyword matching within a session.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `session_id` | string | Yes | - | The unique identifier of the session to search |
| `query` | string | Yes | - | The search query text |
| `limit` | integer | No | 10 | Maximum number of results to return |

**Example Request:**

```json
{
  "session_id": "session_abc123",
  "query": "What was discussed about Rust programming?",
  "limit": 5
}
```

**Response:**

```json
{
  "results": [
    {
      "id": "turn_xyz789",
      "score": 0.89,
      "content": "We discussed Rust's ownership model...",
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

#### hippos_semantic_search

Perform pure semantic (vector-based) search within a session.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `session_id` | string | Yes | - | The unique identifier of the session to search |
| `query` | string | Yes | - | The semantic search query |
| `limit` | integer | No | 10 | Maximum number of results to return |

**Example Request:**

```json
{
  "session_id": "session_abc123",
  "query": "How does async programming work?",
  "limit": 5
}
```

**Response:**

```json
{
  "results": [
    {
      "id": "turn_abc456",
      "score": 0.92,
      "content": "Async programming in Rust uses the async/await syntax...",
      "metadata": {
        "turn_number": 10,
        "created_at": "2024-01-15T14:00:00Z"
      }
    }
  ],
  "total_results": 1,
  "search_time_ms": 20
}
```

### Testing with MCP Inspector

You can test the MCP server using the official MCP Inspector tool:

```bash
# Install MCP Inspector
npx @modelcontextprotocol/inspector

# Run inspector against your MCP server
npx @modelcontextprotocol/inspector ./target/release/hippos
```

### Client Integration

#### Claude Desktop

Add Hippos to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "hippos": {
      "command": "/full/path/to/hippos",
      "args": [],
      "env": {
        "HIPPOS_MCP_MODE": "1"
      }
    }
  }
}
```

#### Cursor IDE

Add to your Cursor MCP configuration:

```json
{
  "mcpServers": {
    "hippos": {
      "command": "/full/path/to/hippos",
      "args": [],
      "env": {
        "HIPPOS_MCP_MODE": "1"
      }
    }
  }
}
```

#### Claude Code CLI

```json
{
  "mcpServers": {
    "hippos": {
      "command": "/full/path/to/hippos",
      "args": [],
      "env": {
        "HIPPOS_MCP_MODE": "1"
      }
    }
  }
}
```

> **Note**: Replace `/full/path/to/hippos` with the actual path to your compiled binary.

### Environment Variables for MCP Mode

| Variable | Default | Description |
|----------|---------|-------------|
| `HIPPOS_MCP_MODE` | `0` | Set to `1` to enable MCP stdio server mode |
| `EXOCORTEX_DATABASE_URL` | `ws://localhost:8000` | SurrealDB connection URL |
| `EXOCORTEX_API_KEY` | `dev-api-key` | API key for authentication |

## 🔒 Security

### Authentication Methods

#### API Key Authentication

Simple token-based authentication suitable for service-to-service communication:

```bash
curl -H "Authorization: ApiKey YOUR_API_KEY" http://localhost:8080/api/v1/sessions
```

**Configuration:**

```yaml
security:
  api_key: "your-secret-api-key"
```

#### JWT Authentication

Bearer token authentication using JSON Web Tokens:

```bash
curl -H "Authorization: Bearer YOUR_JWT_TOKEN" http://localhost:8080/api/v1/sessions
```

**JWT Claims Structure:**

```json
{
  "sub": "user_id",
  "tenant_id": "tenant_1",
  "role": "admin",
  "exp": 1705315200,
  "iss": "hippos",
  "aud": "hippos-api"
}
```

**Configuration:**

```yaml
security:
  jwt_secret: "your-32-character-secret-key"
  jwt_issuer: "hippos"
  jwt_audience: "hippos-api"
  jwt_expiry_seconds: 3600
```

### Rate Limiting

Hippos implements Token Bucket rate limiting:

| Limit Type | Description | Default |
|------------|-------------|---------|
| Global | Requests per minute globally | 1000/min |
| Per-Session | Requests per minute per session | 100/min |
| Per-Endpoint | Custom limits per endpoint | Configurable |

**Configuration:**

```yaml
security:
  rate_limit_enabled: true
  global_rate_limit: 1000
  per_session_rate_limit: 100
  redis_url: "redis://localhost:6379"
```

### Role-Based Access Control (RBAC)

Predefined roles:

| Role | Permissions |
|------|-------------|
| `admin` | Full access to all resources |
| `user` | Access to own resources |
| `readonly` | Read-only access |

**Custom RBAC Configuration:**

```rust
use hippos::security::rbac::{Role, Permission, Resource};

// Define custom permissions
let permissions = vec![
    Permission::new("sessions:read", Role::User),
    Permission::new("sessions:write", Role::User),
    Permission::new("sessions:delete", Role::Admin),
];
```

### Request Validation

All incoming requests are validated:

- **JSON Schema Validation**: Request body structure
- **Type Validation**: Field types and formats
- **Size Limits**: Maximum request body size
- **Content-Type**: Required content type for POST/PUT

## 🛠️ Development

### Building from Source

```bash
# Debug build
cargo build

# Release build with optimizations
cargo build --release

# Build with specific features
cargo build --release --features "metrics,security"
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test --lib index

# Run with output
cargo test --lib -- --nocapture

# Run integration tests
cargo test --test integration
```

### Test Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

### Adding New Features

#### 1. Create a New Module

```rust
// src/new_feature/mod.rs
pub mod handler;
pub mod service;
pub mod model;
```

#### 2. Define Data Models

```rust
// src/new_feature/model.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NewFeature {
    pub id: String,
    pub name: String,
    // Add fields
}
```

#### 3. Implement Service

```rust
// src/new_feature/service.rs
use async_trait::async_trait;

#[async_trait]
pub trait NewFeatureService {
    async fn create(&self, input: Input) -> Result<Output>;
    async fn get(&self, id: &str) -> Result<Output>;
}
```

#### 4. Create Handler

```rust
// src/new_feature/handler.rs
use axum::{Json, extract::State};
use crate::api::AppState;

pub async fn create_feature(
    State(state): State<AppState>,
    Json(request): Json<CreateRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Implementation
}
```

#### 5. Add Routes

```rust
// src/api/routes/new_feature_routes.rs
pub fn create_new_feature_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_feature))
        .route("/:id", get(get_feature))
}
```

#### 6. Register Router

```rust
// src/api/mod.rs
pub fn create_router(app_state: AppState) -> Router {
    let api = Router::new()
        .merge(routes::session_routes::create_session_router())
        .merge(routes::turn_routes::create_turn_router())
        .merge(routes::search_routes::create_search_router())
        .merge(routes::new_feature_routes::create_new_feature_router()); // Add here

    Router::new().nest("/api/v1", api).with_state(app_state)
}
```

### Code Style

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Lint code
cargo clippy

# Fix clippy suggestions
cargo clippy --fix
```

### Database Migrations

```bash
# Run migrations
cargo run -- migrate

# Create new migration
cargo run -- migration create migration_name
```

### Performance Benchmarking

```bash
# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench search_latency
```

## 📦 Dependencies

### Core Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| `axum` | 0.7 | Web framework |
| `surrealdb` | 1.0 | Database |
| `tokio` | 1.35 | Async runtime |
| `tracing` | 0.1 | Structured logging |
| `serde` | 1.0 | Serialization |

### Optional Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| `candle-core` | 0.4 | ML/embedding inference |
| `tokenizers` | 0.22 | Text tokenization |
| `redis` | 0.25 | Rate limiting |
| `jsonwebtoken` | 10.2 | JWT authentication |
| `openssl` | 0.10 | Cryptography |

## 🤝 Contributing

### Getting Started

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit changes: `git commit -m 'Add amazing feature'`
4. Push to branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

### Development Workflow

1. **Read**: Read existing code and documentation
2. **Understand**: Understand the architecture and patterns
3. **Implement**: Write clean, tested code
4. **Test**: Ensure all tests pass
5. **Document**: Update documentation as needed
6. **Review**: Address review feedback

### Code Standards

- Follow Rust best practices (rustfmt, clippy)
- Write comprehensive tests
- Document public APIs with doc comments
- Use meaningful variable and function names
- Keep functions small and focused
- Write descriptive commit messages

### Pull Request Guidelines

- Provide clear description of changes
- Link related issues
- Include test coverage
- Update documentation
- Ensure CI passes

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [SurrealDB](https://surrealdb.com/) - Database
- [Axum](https://github.com/tokio-rs/axum) - Web Framework
- [Candle](https://github.com/huggingface/candle) - ML Framework
- [Tokio](https://tokio.rs/) - Async Runtime

## 📞 Support

- **Documentation**: [docs.hippos.io](https://docs.hippos.io)
- **Issues**: [GitHub Issues](https://github.com/hippos/hippos/issues)
- **Discussions**: [GitHub Discussions](https://github.com/hippos/hippos/discussions)

---

**Hippos** - Empowering AI Agents with Persistent Memory
