---
kind: fix
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

`handle_redirect` in `src/web.rs` accepts a token response with no ID token: it logs "nonce validation skipped" and signs the user in with only an access token. The SDK always sends the `openid` scope, and OpenID Connect requires an ID token in that response, so a missing one means a misbehaving or spoofed token endpoint, and skipping the nonce check weakens the AUTH-T1 mitigation in `docs/threat.toml`. Return `AuthError::IdTokenValidationFailed("no ID token in the token response")` instead, and test it natively: `web.rs` tests are only built for wasm32, never run, so put the "is a missing ID token an error" decision in a pure function outside `web` and test that.
