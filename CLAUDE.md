# Shuriken SDK (Rust)

Public SDK for the Shuriken API (REST + WebSocket streams). Version 0.3.0.

## After every change

1. **Bump the version** in `Cargo.toml` following semver:
   - **patch** (0.3.0 -> 0.3.1): bug fixes, internal refactors
   - **minor** (0.3.0 -> 0.4.0): new streams, new types, new API methods
   - **major** (0.3.0 -> 1.0.0): breaking changes (renamed types, removed streams, changed method signatures)
2. **Update `README.md`** if the change affects the public API surface.

## Build & check

- Check: `cargo check` (REST only) / `cargo check --features ws` (with WebSocket)
- Test: `cargo test` / `cargo test --features ws`
- Lint: `cargo clippy -- -D warnings` / `cargo clippy --features ws -- -D warnings`
- Format: `cargo fmt --check`

## Architecture

- `src/http/mod.rs` — `ShurikenHttpClient` (Clone) with namespace accessors: `.account()`, `.tokens()`, `.swap()`, `.portfolio()`, `.perps()`, `.trigger()`
- `src/http/account.rs`, `src/http/tokens.rs`, etc. — REST API namespace modules, each exposing an `XxxApi<'_>` struct
- `src/error.rs` — `ShurikenError` enum
- `src/ws/mod.rs` — `ShurikenWsClient` (separate from HTTP client), `ConnectionState`, `ConnectionStateEvent`
- `src/ws/streams.rs` — `StreamDef<P, F>` type, filter types (`SvmTokenFilter`, `EvmTokenFilter`, etc.), 21 typed stream constants
- `src/ws/subscription.rs` — `Subscription<T>` implementing `futures_core::Stream`
- `src/ws/connection.rs` — Internal Pusher protocol handling
- Stream payload types come from the `shuriken-api-types` crate (separate repo)

## Key design decisions

- `ShurikenHttpClient` is `Clone` (wraps `reqwest::Client` + `Arc`-ed config) — share freely across tasks
- `ShurikenWsClient` is a separate type, not embedded in the HTTP client — it manages its own mutable connection state
- WebSocket is gated behind the `ws` cargo feature to keep the default dependency footprint small
- Typed streams use `StreamDef<P, F>` constants that pair a payload type `P` with a filter type `F`, ensuring compile-time correctness of subscribe calls
- `Subscription<T>` implements `Stream` so callers use `StreamExt::next()` rather than callbacks

## Dependencies

- `shuriken-api-types` provides all stream payload types and is published to crates.io
- WebSocket deps (`tokio-tungstenite`, `futures-util`, `futures-core`, `tracing`) are optional behind the `ws` feature

## Type sync

- REST API response types are defined in `src/http/*.rs` and must match the backend API contract
- Stream event types live in `shuriken-api-types` and must stay in sync with the TypeScript SDK's `src/streams/` types and the Rust types in the private monorepo
