# Wardnet Daemon - Rust Engineer Memory

## IMPORTANT: Memory File Location
This memory file lives at the **repo root**: `.claude/agent-memory/rust-engineer/MEMORY.md`
NOT inside `source/daemon/`. Always read and update memory at the repo root, regardless of working directory.

## Repository Module Structure (Refactored)
- **Traits** live in `src/repository/<name>.rs` (e.g. `device.rs`, `tunnel.rs`)
- **SQLite implementations** live in `src/repository/sqlite/<name>.rs`
- `src/repository/mod.rs` re-exports both traits and SQLite structs
- `src/repository/sqlite/mod.rs` re-exports all `Sqlite*Repository` types

## Key Patterns
- `replace_all` on identifiers like `WireGuard` is dangerous -- it replaces in code identifiers too, not just doc comments. Only use targeted edits for doc comment fixes.
- sqlx for SQLite maps INTEGER columns to `i64`. When the domain type is `u16` (e.g. listen_port), use `u16::try_from()` at the DB boundary. For insert, sqlx `.bind()` accepts `Option<u16>` directly.
- `TunnelRow.listen_port` is `Option<u16>` (not `i64`) so the service can pass values from parsed config without casting.
- Clippy requires backticks around `WireGuard` in doc comments (`///`) but not in regular comments (`//`).

## Test Conventions
- Tests go in separate files: `src/repository/tests/<name>.rs`, `src/service/tests/<name>.rs`
- Repository tests use `super::test_pool()` (in-memory SQLite with migrations)
- Service tests use hand-written mock structs implementing repository traits (no mocking library)
- Drop `MutexGuard`s before `.await` points in tests to avoid `clippy::await_holding_lock`
