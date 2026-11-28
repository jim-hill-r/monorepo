---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Management app / Phase 3: the server check"
order: 30
added: 2026-11-28
---

Add a Pages Function in Rust (`workers-rs`) on the management app's origin that verifies the Auth0 access token (signature against Auth0's keys, issuer, audience, expiry) and the `manage:cookbook` permission on every `/api/*` call, and returns 401 or 403 otherwise. This is the real gate, not the app's pages. Write it so decision 0018's flags Worker can reuse it, add tests for each rejection, and an eval that an unsigned call is refused. Needs [manage-app-shell](manage-app-shell.md). See the [roadmap](../roadmap.md).
