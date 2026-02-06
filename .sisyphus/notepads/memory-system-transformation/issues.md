# WebSocket Compilation Fixes

## Date
2026-02-03

## Issue Summary
Fixed 2 compilation errors preventing `cargo check` from passing.

## Errors Fixed

### Error 1 (E0432): `axum::extract::ws` not found
- **Location**: `src/websocket/mod.rs:8`
- **Cause**: The `ws` feature was not enabled for axum in Cargo.toml
- **Fix**: Added `ws` feature to axum dependency in Cargo.toml line 23

### Error 2 (E0282): Type annotations needed for sender
- **Location**: `src/websocket/mod.rs:40`
- **Cause**: Cannot infer type for `sender` field in `WebSocketConnection` struct
- **Fix**: Changed sender type from `SplitSink<WebSocketStream, Message>` to `SplitSink<WebSocket, Message>`

## Additional Fixes Required
Enabling the `ws` feature introduced additional compilation issues:

### Issue 1: Missing StreamExt trait
- **Fix**: Added `use futures_util::StreamExt;` import alongside existing `SinkExt`

### Issue 2: connection_id moved multiple times
- **Location**: `src/websocket/mod.rs:125-131`
- **Fix**: Created separate clones (`connection_id_for_receive`, `connection_id_for_forward`) before moving into closures

### Issue 3: parking_lot::Mutex not Send
- **Problem**: `parking_lot::MutexGuard` contains raw pointers and is not `Send`, causing `tokio::spawn` to fail
- **Fix**: Switched from `parking_lot::Mutex` to `tokio::sync::Mutex` throughout the websocket module

### Issue 4: Async lock() requires await
- **Problem**: `tokio::sync::Mutex::lock()` returns a Future, not a guard directly
- **Fix**: Added `.await` to all `connection.lock()` calls

### Issue 5: Future borrowing from guard
- **Location**: `src/websocket/mod.rs:330-335`
- **Problem**: `sender.send()` returns a Future that borrows from the MutexGuard
- **Fix**: Restructured code to hold lock across the await for the send operation

## Files Modified
1. `Cargo.toml` - Added `ws` feature to axum
2. `src/websocket/mod.rs` - Multiple refactoring for async compatibility

## Verification
- Ran `cargo check 2>&1`
- Result: Compilation succeeds with only warnings (no errors)

---

# Memory System Compilation Fixes

## Date
2026-02-03

## Issue Summary
Fixed 5 compilation errors preventing `cargo test --lib` from passing.

## Errors Fixed

### Error 1 (E0599): `RelationshipType::DEPENDS_ON` not found
- **Location**: `src/models/entity.rs:660`
- **Cause**: Used wrong variant name (SCREAMING_SNAKE_CASE instead of PascalCase)
- **Fix**: Changed `RelationshipType::DEPENDS_ON` to `RelationshipType::DependsOn`

### Error 2 (E0308): Mismatched types in verify_fact
- **Location**: `src/models/profile.rs:474`
- **Cause**: `add_fact()` method returns `()` but test expected it to return `fact_id`
- **Fix**: Modified `add_fact()` to return `String` (the fact_id) instead of `()`

### Error 3 (E0308): Mismatched types for embedding field
- **Location**: `src/services/pattern_manager.rs:880`
- **Cause**: `embedding` field expects `Option<Vec<f32>>` but `Vec::new()` was passed without wrapping in Some
- **Fix**: Changed `embedding: Vec::new()` to `embedding: Some(Vec::new())`

### Error 4 (E0063): Missing fields in Memory struct initializer
- **Location**: `src/services/pattern_manager.rs:873`
- **Cause**: Memory struct initialization missing required fields: `accessed_at`, `confidence`, `expires_at`, `full_summary`, `parent_id`, `related_ids`, `source`, `source_id`, `tags`, `topics`, `keywords`
- **Fix**: Added all missing fields with appropriate default values:
  - `confidence: 0.9`
  - `source: MemorySource::Conversation`
  - `source_id: None`
  - `parent_id: None`
  - `related_ids: vec![]`
  - `tags: vec![]`
  - `topics: vec![]`
  - `accessed_at: Utc::now()`
  - `expires_at: None`
  - `full_summary: None`
  - `keywords: vec![]`

### Error 5 (E0596): Cannot borrow `relationship` as mutable
- **Location**: `src/models/entity.rs:650`
- **Cause**: Variable not declared as mutable when trying to call `.update()` method
- **Fix**: Changed `let relationship` to `let mut relationship`

## Files Modified
1. `src/models/entity.rs` - Fixed DEPENDS_ON variant name and added mut keyword
2. `src/models/profile.rs` - Modified add_fact() to return fact_id
3. `src/services/pattern_manager.rs` - Fixed embedding type and added Memory fields

## Verification
- Ran `cargo test --lib 2>&1 | grep "^error\["`
- Result: Zero compilation errors (only warnings remain)
- 91 tests passed, 2 tests failed (test logic issues, not compilation)
