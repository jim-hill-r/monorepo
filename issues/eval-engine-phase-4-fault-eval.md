---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Eval engine
order: 130
added: 2026-11-26
---

Eval engine phase 4. Add the `fault` eval kind and the browser fault catalogue (`offline`, `slow-3g`, `asset-500`, `auth-down`, `third-party-blocked`, `storage-denied`) as Playwright helpers, and `nightly-chaos.yml` (3:30 AM Pacific) that runs `chaos` evals only for projects whose `cd` evals passed that night, only when the repo variable `EVALS_CHAOS` is `on`, after `free-tiers.sh usage` passes, with one browser per project and a hard cap of 500 Worker requests a run.
