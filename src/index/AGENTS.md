# Index Layer

**Type:** Search infrastructure (vector + full-text + embeddings)

## OVERVIEW
Hybrid search engine with vector similarity, full-text search, and embedding generation (Candle ML framework).

## STRUCTURE
```
index/
├── mod.rs
├── embedding/    # Transformer embeddings (Candle)
├── vector/       # Vector similarity search
└── full_text/    # Full-text search
```

## WHERE TO LOOK
| Task | Subdir |
|------|--------|
| Embeddings | `embedding/` |
| Vector search | `vector/` |
| Full-text search | `full_text/` |
| Index coordination | `mod.rs` |

## CONVENTIONS
- Uses `DashMap` for in-memory vector index
- Candle framework for local embedding inference
- RRF (Reciprocal Rank Fusion) for hybrid results

## ANTI-PATTERNS (THIS MODULE)
- ❌ None significant - well-structured modular design
