---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Eval engine
order: 30
added: 2026-11-26
---

Eval engine phase 1. Make `evals publish` commit each run's results to the long-lived `evals-results` data branch (`results/<project>/<yyyy-mm>.jsonl` appended, `verdicts.json` recomputed, one commit per run, retry on a push race) with `contents: write` for that step only, add `evals verdicts`, and a yearly step that archives months older than 12 to `results/<project>/archive/<yyyy>.jsonl.gz`. No database. Add a repo check that eval workflows push only to `evals-results`, and note in `all-delete-branches.yml` that the branch must never be deleted.
