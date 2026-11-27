---
added: 2026-11-27
runbook: standards/docs/runbooks/eval-triage-task.md
---

## Requirement

Add a Claude scheduled task, "Daily eval triage", in the Monorepo project: every day at 7:30 AM New York time (4:30 AM Pacific), with the prompt in `standards/docs/scheduled-tasks/eval-triage.md`. It runs the new `eval-triage` skill, which files a `FIX` for each must requirement not met in production, a `TODO` for each broken eval, and rollback steps for a failed deploy, in one draft PR. Cost: $0 extra, inside the Claude Code subscription; a quiet day takes a few minutes.

Eval engine phase 2, standards/docs/plans/2026-11-eval-engine.md ("Triage").

## Why it can't be automated

Claude scheduled tasks are made in the claude.ai project settings, which have no API, and agents must not schedule recurring work that spends the Claude budget only Jim controls.

## Runbook

[Schedule the daily eval triage](../../standards/docs/runbooks/eval-triage-task.md)
