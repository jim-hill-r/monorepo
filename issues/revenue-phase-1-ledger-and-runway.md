---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Revenue / Phase 1: the financial ledger and runway"
order: 10
added: 2026-11-27
---

Revenue phase 1 (standards/docs/plans/2026-11-revenue-402.md). Use Jim's answers in decision 0012. Add `standards/money/runway.toml` (start month, opening balance, thresholds) and a money model in `cost/model` (balance, three-month burn, runway, status `funded`, `low` or `out`) with unit tests for no revenue, a negative balance and zero burn; a `cost money` command that writes `runway.json` and the monthly statement `standards/money/statements/<yyyy-mm>.json`; run it in the daily usage workflow (cost tracking phase 1) and upload the JSON; document the files in `standards/docs/money.md`. $0, no new vendor.
