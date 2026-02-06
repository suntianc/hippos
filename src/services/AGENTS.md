# Services Layer

**Type:** Business logic controllers

## OVERVIEW
Domain services implementing core business logic for memory management, pattern management, and session lifecycle.

## STRUCTURE
```
services/
├── mod.rs
├── dehydration.rs      # Content summarization
├── retrieval.rs        # Search coordination
├── pattern_manager.rs  # Pattern CRUD + matching (58KB)
├── performance.rs      # Performance metrics
├── memory_builder.rs   # Memory construction
├── memory_recall.rs    # Memory retrieval
├── memory_integrator.rs # Memory integration
├── entity_manager.rs   # Entity management
├── profile_manager.rs  # Profile management
├── session/            # Modular subdir
│   └── mod.rs
└── turn/               # Modular subdir
    └── mod.rs
```

## WHERE TO LOOK
| Task | File |
|------|------|
| Search logic | `retrieval.rs`, `memory_recall.rs` |
| Pattern operations | `pattern_manager.rs` |
| Session management | `session/` |
| Turn management | `turn/` |
| Memory operations | `memory_builder.rs`, `memory_integrator.rs` |

## CONVENTIONS
- Large files (>25KB): `pattern_manager.rs`, `performance.rs`, `entity_manager.rs`
- Async functions return `Result<T, AppError>`
- Use `async-trait` for trait implementations

## ANTI-PATTERNS (THIS MODULE)
- ❌ Mix of flat files and modular subdirs (`session/`, `turn/` vs flat)
- ❌ `performance.rs` named singular, others use `_manager` or `_builder`
- ❌ Very large files not split further
