---
kind: fix
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

`handle_redirect` in `src/web.rs` accepts a token response with no ID token: it logs "nonce validation skipped" and signs the user in with only an access token. The SDK always sends the `openid` scope, and OpenID Connect requires an ID token in that response, so a missing one means a misbehaving or spoofed token endpoint, and skipping the nonce check weakens the AUTH-T1 mitigation in `docs/threat.toml`. Return `AuthError::IdTokenValidationFailed("no ID token in the token response")` instead, and add the case to the module's tests once they compile for wasm32 (see `web-tests-dont-compile-for-wasm32.md`).
