---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Revenue / Phase 1: the financial ledger and runway"
order: 20
added: 2026-11-28
---

Run `cost money` (standards/docs/money.md) on a schedule: daily in the usage workflow from cost tracking phase 1 (issues/cost-tracking-phase-1-daily-usage.md), uploading `runway.json` and the month-to-date statement as the run's artifact, and on the 1st of each month a run that writes the closed month with `--out standards/money` and opens a PR, as the revenue plan says (cost/docs/plans/2026-11-revenue-402.md, "The financial ledger"). Both checkouts need `fetch-depth: 0`, since the merged PRs split the shared costs. $0: GitHub Actions minutes on the free plan, no new vendor or credential.
