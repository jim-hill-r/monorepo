---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 10
added: 2026-11-26
---

Threat check phase 1. Add `standards/docs/security.md` with the standards in the plan (no non-public data on a public surface, previews included; Auth0 authorization code with PKCE and no client secret in client builds; required security headers on every public web surface; secrets only in GitHub or Worker secrets with the narrowest scope; least-privilege workflows with commit-pinned third-party actions), link it from `standards/docs/README.md`, and add a root `SECURITY.md` that points reporters to GitHub private vulnerability reporting.
