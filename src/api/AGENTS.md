# API Layer

**Type:** REST API handlers, routes, DTOs

## OVERVIEW
Axum-based HTTP layer with handlers, routes, and DTOs for all domain operations.

## STRUCTURE
```
api/
├── mod.rs           # Router assembly
├── app_state.rs     # Application state (SHOULD MOVE)
├── handlers/        # Request handlers (8 files)
│   ├── session_handler.rs
│   ├── turn_handler.rs
│   ├── memory_handler.rs
│   ├── search_handler.rs
│   ├── pattern_handler.rs
│   ├── entity_handler.rs
│   └── profile_handler.rs
├── routes/          # Route definitions
│   ├── session_routes.rs
│   ├── turn_routes.rs
│   ├── memory_routes.rs
│   └── ...
└── dto/             # Request/Response DTOs (7 files)
    ├── session_dto.rs
    ├── turn_dto.rs
    └── ...
```

## WHERE TO LOOK
| Task | Location |
|------|----------|
| Add endpoint | `routes/` + `handlers/` + `dto/` |
| Modify request body | `dto/` |
| Change response format | `dto/` |

## CONVENTIONS
- DTOs in `dto/` with `_dto.rs` suffix
- Handlers return `Result<impl IntoResponse, AppError>`
- Use `Json<T>` extractor for request bodies

## ANTI-PATTERNS (THIS MODULE)
- ❌ `app_state.rs` at root instead of grouped
- ❌ Inconsistent naming: `session_routes.rs` vs `pattern_routes.rs` (missing `_routes` on some)
