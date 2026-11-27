---
kind: fix
priority: priority
tags: [agent-generated]
added: 2026-11-27
---

The tests in `src/web.rs` don't compile, and CI never notices. `web` is built only for `wasm32`, and `mise run ci` runs `cargo test` for the host, so these tests are never built. Four of them call `WebAuthProvider::create_client_from_explicit_endpoints`, which no longer exists (the explicit-endpoint path was removed when openidconnect v4 made discovery required): `cargo test --no-run --target wasm32-unknown-unknown` fails with four `E0599` errors. Delete or rewrite those tests, then make CI at least build the wasm32 tests (`ci:deps` adding the target and a `cargo clippy --target wasm32-unknown-unknown --all-targets -- -D warnings` step in `auth_sdk/mise.toml`) so `web.rs` can't rot again. Pure logic that needs real tests belongs in `redirect.rs`, which tests natively.
