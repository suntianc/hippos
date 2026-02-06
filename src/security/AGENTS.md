# Security Layer

**Type:** Authentication, authorization, validation

## OVERVIEW
Multi-layer security: API key auth, JWT validation, RBAC, rate limiting, request validation.

## WHERE TO LOOK
| Task | File |
|------|------|
| API key auth | `auth.rs` |
| JWT handling | `auth.rs` (jsonwebtoken crate) |
| RBAC policies | `rbac.rs` |
| Rate limiting | `rate_limit.rs` (Redis-backed) |
| Request validation | `validation.rs` + `middleware.rs` |
| Security config | `config.rs` |

## CONVENTIONS
- Development/production methods: `Auth::development()`, `Auth::production()`
- Rate limiting uses token bucket algorithm
- JWT claims include tenant_id, role

## ANTI-PATTERNS (THIS MODULE)
- ❌ None significant - well-organized
