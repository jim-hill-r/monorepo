---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Review dashboard epic / Phase 7: revenue and cost"
order: 320
added: 2026-11-26
---

Read every `docs/cost.toml` (through `cost_model::parse_cost_file`), the radar's monthly budgets (`standards/radar/radar.json`) and the limits in `standards/radar/limits.toml` into the snapshot in `pane_snapshot`: per project its services, plans, money this month and expected usage, and account-wide expected usage against each budget. Fail the build on an invalid file, with fixture tests.
