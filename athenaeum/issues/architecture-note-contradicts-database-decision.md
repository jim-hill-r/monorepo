---
kind: fix
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

`resources/apps/architecture.md` tells engineers to use the database as the
only API and send JWTs straight to SurrealDB or SpacetimeDB. That contradicts
the repo's rules: "Files before databases" in `AGENTS.md`, and
[decision 0004](../../standards/docs/decisions/0004-database-engine.md),
which picks committed SQLite and then Cloudflare D1. Neither SurrealDB nor
SpacetimeDB is an approved vendor or on the tech radar.

Rewrite the note to match the current architecture (static clients, Auth0
sign-in through `auth_sdk`, files then SQLite then D1), linking the decision
and the tech radar instead of restating them, or mark the old idea as a
superseded note.
