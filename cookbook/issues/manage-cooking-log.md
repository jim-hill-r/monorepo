---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Management app / Phase 4: the cooking log"
order: 40
added: 2026-11-28
---

Let a curator log "we cooked this" for a recipe in the management app: the date, who cooked (from the token), a rating, notes and what they changed, and see every past cook of a recipe. Store it where Jim's open decision in the [roadmap](../docs/roadmap.md) says (recommended: D1, since the notes name people and the repo is public), through [the server check](manage-api-token-check.md). Add the D1 database to the `infra` cloudflare stack and `docs/cost.toml`. Logging a cook can move a recipe from `draft` to `tested`.
