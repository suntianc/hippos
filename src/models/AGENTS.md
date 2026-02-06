# Domain Models

**Type:** Domain entities and repositories

## OVERVIEW
Core domain objects for sessions, turns, memories, patterns, entities, and profiles.

## WHERE TO LOOK
| Entity | File | Notes |
|--------|------|-------|
| Session | `session.rs` | Tenant-aware, status tracking |
| Turn | `turn.rs` | Conversation turn with metadata |
| Memory | `memory.rs` | Indexed content with importance |
| Pattern | `pattern.rs` | Knowledge patterns (5 types) |
| Entity | `entity.rs` | Named entities with metadata |
| Profile | `profile.rs` | User preferences |

## REPOSITORY PATTERN
- `*_repository.rs` files implement storage traits
- `memory_repository.rs`, `entity_repository.rs`, `profile_repository.rs`, `pattern_repository.rs`
- Abstract over SurrealDB/ArangoDB

## CONVENTIONS
- `snake_case` file names
- `PascalCase` types
- `#[derive(Serialize, Deserialize)]` for all structs
- Use `sqlx` or `surrealdb` query builders

## ANTI-PATTERNS (THIS MODULE)
- ❌ `session.rs` and `turn.rs` not in repositories (unlike memory, entity)
- ❌ Repository pattern inconsistent across entities
