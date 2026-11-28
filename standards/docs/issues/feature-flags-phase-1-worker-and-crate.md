---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Feature flags / Phase 1: flags Worker and crate"
order: 10
added: 2026-11-27
---

Build the `jimhillr-flags` Worker (workers-rs, `d1` feature) and the D1 schema (`modes`, `settings`, `users`, append-only `history` with triggers) and the `flags` crate, as items 1 to 5 of [decision 0018](../decisions/0018-feature-flags.md) say: Auth0 access token and `manage:{app}` permission checks, the catalog route for deploys, batch writes with history, stale-write 409s, defaults on failure. Include the Worker's `docs/threat.toml` (the threat table in 0018), `docs/cost.toml`, requirements and evals.
