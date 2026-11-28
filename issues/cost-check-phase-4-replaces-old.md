---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Cost check
order: 100
added: 2026-11-26
---

Cost check phase 4 (replaces the old cost ledger and monthly review TODOs). Make the weekly usage run compare actuals with the projects' expected usage and write `standards/cost/usage/<yyyy-mm>.json`, plus an `observe` result for each `cost` eval (met when the actuals are within the expected usage plus a margin; see "Cost evals" in `standards/docs/evals.md`); add `cost render --ledger` for a generated ledger; add a `review-costs` agent skill that labels a PR `needs-human` when anything is above 80% of a free limit or not $0, and ask Jim to schedule it monthly.
