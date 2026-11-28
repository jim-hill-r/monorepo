---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Eval engine
added: 2026-11-27
---

Give TELE-004 (no personal details are stored) a production-stage eval. The `http` evals can't see what is stored, and eval traffic is dropped before it is read, so it needs the `metric` kind (`evals/docs/issues/eval-engine-phase-3-metric-eval.md`): an `observe` eval over the `telemetry` Analytics Engine dataset that finds no note, error message or CLI error matching an email address or phone number in the last 7 days.
