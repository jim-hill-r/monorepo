---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Evals roadmap / Phase 4: Stress and judgment"
order: 40
added: 2026-11-26
---

Add the signed `x-jimhillr-chaos` header (HMAC with `EVALS_CHAOS_KEY`, short expiry, ignored when invalid) to the `telemetry` Worker and pane's Worker, with the `api-error`, `api-slow` and `upstream-down` faults, and ask Jim to create the secret.
