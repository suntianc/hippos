# HIPPOS KNOWLEDGE BASE

**Generated:** 2026-02-03
**Branch:** (current)
**Type:** Rust Context Management Service (Axum + SurrealDB)

## OVERVIEW
High-Performance Context Management Service for AI Agents. Provides persistent conversation memory, hybrid search (vector + full-text), pattern management, and MCP protocol support.

## STRUCTURE
```
hippos/
├── src/
│   ├── api/          # REST handlers, routes, DTOs
│   ├── models/       # Domain entities + repositories
│   ├── services/     # Business logic (memory, pattern, session, turn)
│   ├── storage/      # SurrealDB/ArangoDB abstraction
│   ├── index/        # Vector, full-text, embedding layers
│   ├── security/     # Auth (API key + JWT), RBAC, rate limiting
│   ├── mcp/          # Model Context Protocol server
│   └── websocket/    # Real-time subscriptions
├── Cargo.toml
├── config.yaml
├── test_api.sh       # Integration tests
└── test_mcp_server.sh
```

## WHERE TO LOOK
| Task | Location |
|------|----------|
| Add API endpoint | `src/api/routes/` + `src/api/handlers/` |
| New domain model | `src/models/` + repository trait |
| Business logic | `src/services/` |
| Database changes | `src/storage/` |
| Security/Auth | `src/security/` |
| Search functionality | `src/index/` |

## COMMANDS
```bash
cargo build --release          # Build
cargo test --lib               # Unit tests
./test_api.sh                  # Integration tests
cargo run                      # REST API (default)
HIPPOS_MCP_MODE=1 cargo run    # MCP stdio mode
```

## CONVENTIONS
- **Naming**: `snake_case` for files, `PascalCase` for types
- **Async**: All async functions use `tokio`
- **Error handling**: `anyhow` for errors, `thiserror` for typed errors
- **Testing**: `#[cfg(test)]` inline modules + `rstest` fixtures

## ANTI-PATTERNS (THIS PROJECT)
- ❌ Orphaned empty dirs: `src/metrics/`, `src/utils/`
- ❌ Mixed storage abstraction: `surrealdb.rs` + `arangodb.rs` at same level
- ❌ Inconsistent module structure: `session/`, `turn/` are modules but others are flat files
- ❌ `app_state.rs` at `api/` root level instead of grouped

## SECURITY
- API Key: `dev-api-key` (change in production)
- JWT: 32+ char secret required
- Rate limiting: Redis-backed, token bucket

## DEPLOYMENT
- Default port: `8080`
- Config: `config.yaml` with env var overrides
- Database: SurrealDB HTTP API (not WebSocket SDK)
