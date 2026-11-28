---
kind: fix
priority: priority
tags: [agent-generated]
epic: "Evals roadmap / Phase 1: Trust the green"
order: 10
added: 2026-11-28
---

The coverage line counts a must as "evaluated in production" when it has an eval with a production stage, not when one has a production result. `musts_in_production` in `evals/cli/src/store.rs` (around line 227) looks only at `evals.toml`, so cookbook said "3 of 3 musts evaluated in production" while COOK-005's only `cd` eval had never run (finding 5 of the [fire drill](../docs/fire-drill-2026-11-27.md)). Count a must only when one of its production-stage evals has a stored `pass` or `fail` result inside the freshness window of `verdicts.toml`, keep the old count as `musts_with_production_evals` so the gap shows, use the new count in the badges, and add a test for a must whose `cd` eval never ran. Part of the [evals roadmap](../docs/roadmap.md).
