---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Eval engine
order: 55
added: 2026-11-26
---

Eval engine phase 1. With `evals verdicts`, fold the human-check answers in `standards/settings-checks.jsonl` into the verdicts (see "Human checks" in `standards/docs/evals.md`): a requirement whose `human-check` setting was last answered "not ok" is not met, and one whose check is past twice its `every` with no answer is at risk, never met.
