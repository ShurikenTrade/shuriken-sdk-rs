# Shuriken SDK (Rust)

Public SDK for the Shuriken API (REST + WebSocket streams).

## After every change

1. **Bump the version** in `Cargo.toml` following semver:
   - **patch** (0.1.0 → 0.1.1): bug fixes, internal refactors
   - **minor** (0.1.0 → 0.2.0): new streams, new types, new API methods
   - **major** (0.1.0 → 1.0.0): breaking changes (renamed types, removed streams, changed method signatures)
2. **Update `README.md`** if the change affects the public API surface.

## Build & check

- Check: `cargo check` (REST only) / `cargo check --features ws` (with WebSocket)
- Test: `cargo test` / `cargo test --features ws`
- Lint: `cargo clippy -- -D warnings` / `cargo clippy --features ws -- -D warnings`
- Format: `cargo fmt --check`

## Architecture

- `src/client.rs` — `ShurikenClient` with HTTP helpers and optional `ws` field
- `src/error.rs` — `ShurikenError` enum
- `src/api/` — REST API modules (account, tokens, portfolio, swap, perps, trigger)
- `src/ws/` — WebSocket client (Pusher protocol), gated behind `ws` cargo feature
- Stream payload types come from the `shuriken-api-types` crate (separate repo)

## Dependencies

- `shuriken-api-types` is pinned to a git tag — bump the tag in `Cargo.toml` when updating
- WebSocket deps (`tokio-tungstenite`, `futures-util`, `tracing`) are optional behind the `ws` feature

## Type sync

- REST API response types are defined in `src/api/*.rs` and must match the backend API contract
- Stream event types live in `shuriken-api-types` and must stay in sync with the TypeScript SDK's `src/streams/` types and the Rust types in the private monorepo
