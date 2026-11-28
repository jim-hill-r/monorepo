---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Evals roadmap / Phase 1: Trust the green"
order: 50
added: 2026-11-28
---

Review the first week of `evals-triage.yml` runs. On 2026-11-28, 63 of 92 requirements were not green (51 unknown, 11 at risk, 1 not met), and no triage run had filed anything yet. After the first few daily runs, check that each filed issue names the right requirement and work, that PR triage opened and merged the issue PR, that `--limit` spreads the backlog over days instead of flooding one PR, and that no requirement went red only because the runner couldn't reach a site (finding 4 of the [fire drill](../docs/fire-drill-2026-11-27.md)). Fix what's wrong and write a short retro in `evals/docs/`. Part of the [evals roadmap](../docs/roadmap.md).
