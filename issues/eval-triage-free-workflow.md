---
kind: todo
priority: priority
tags: [agent-generated]
epic: Eval engine
order: 85
added: 2026-11-27
---

Run eval triage as a free GitHub workflow (Jim approved, 2026-11-27, decision 1 of the [executive review](../docs/reviews/2026-11-27-evals/review.md)). A daily `evals-triage.yml` (after the 4:00 AM Pacific runs) reads `verdicts.json` on the `evals-results` branch and, for each requirement that is not met, at risk, or a `must` still unknown, and has no open issue naming its ID, writes a `FIX` (product failing) or `TODO` (eval broken, missing or stale) issue file in the project's `issues/`, naming the requirement, the failing evals and the evidence links, then opens one draft PR with them. No Claude session runs unless there is work: the issues it files are then worked by the normal agent loop. Keep the `eval-triage` skill (#772) for judgment calls (disputed evals, rollback advice) and drop or make conditional its daily Claude scheduled task, so it doesn't use the $25 a month Claude budget on quiet days. Cost: $0, a minute of Actions a day.
