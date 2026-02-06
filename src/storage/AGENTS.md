# Storage Layer

**Type:** Database abstraction (SurrealDB + ArangoDB)

## OVERVIEW
Repository pattern with dual database support. Uses SurrealDB by default (HTTP REST API).

## STRUCTURE
```
storage/
├── mod.rs
├── factory.rs          # Connection pool factory
├── repository.rs       # Repository trait definitions
├── surrealdb.rs        # SurrealDB client
├── arangodb.rs         # ArangoDB client
└── arangodb_repository.rs  # ArangoDB implementation
```

## WHERE TO LOOK
| Task | File |
|------|------|
| Add repository method | `repository.rs` trait + impl |
| Database operations | `surrealdb.rs` (HTTP API client) |
| Connection pooling | `factory.rs` + `surrealdb.rs` (SurrealPool) |

## CONVENTIONS
- Trait-based abstraction in `repository.rs`
- Database-specific implementations in separate files
- HTTP REST API for SurrealDB (not WebSocket SDK)

## ANTI-PATTERNS (THIS MODULE)
- ❌ Dual database support creates complexity
- ❌ Inconsistent abstraction levels
- ❌ `arangodb_repository.rs` wraps ArangoDB but no SurrealDB equivalent
- ❌ Repository pattern implementation scattered
