---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Native sign-in / Phase 6: stay signed in"
order: 60
added: 2026-11-27
---

Keep native users signed in across restarts: keep the refresh token from the `offline_access` scope (today it is thrown away), store it in the OS keychain with the `keyring` crate, refresh the access token on start, and delete it on sign out (revoking it at Auth0). Decide this together with `configurable-scopes.md`, since that issue would drop `offline_access` for web apps. Add a threat to `docs/threat.toml` for a stolen refresh token. Plan: auth_sdk/docs/plans/2026-11-native-sign-in.md.
