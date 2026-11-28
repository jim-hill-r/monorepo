---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-28
---

pane/model/src/money.rs keeps its own `split_shared_costs` and `REPO_WIDE`, and pane/snapshot/src/merged.rs its own merged-PR log parser; `cost_model` now has the same split (`cost_model::split_shared_costs`, `REPO_WIDE`) for the financial ledger (standards/docs/money.md). Use cost_model's and delete pane's copies, so the Money page and the ledger can never split a shared cost differently (decision 0016, full attribution). Consider moving the git log parser into cost_model too.
