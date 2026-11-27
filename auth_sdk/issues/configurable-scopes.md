---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

`WebAuthProvider::login` in `src/web.rs` always asks for `openid profile email read write offline_access`. `read` and `write` aren't scopes Auth0 defines for these apps, and `offline_access` asks for a refresh token that the SDK throws away (only the access token and ID token claims are kept), so the apps request more access than they use. Add a `scopes: Vec<String>` field to `ProviderConfig` (defaulting to `openid profile email` in `ProviderConfig::auth0`, once that helper from the Auth0 URLs PR lands) and request only those. Least privilege is part of AUTH-T1 in `docs/threat.toml`; note the change there.
