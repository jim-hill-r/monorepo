---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Eval engine
order: 20
added: 2026-11-26
---

Eval engine phase 1. Make the shared `ci` tasks in `standards/tasks/` save their reports (`cargo test` output through `tee`, Playwright's JSON reporter), add an `evals` task to each shared task file that passes them to `evals run --stage ci`, and run it after `ci` in `mise-projects.yml` (PR CI, push to main, nightly CI), followed by `evals publish` and an `evals-results.jsonl` artifact upload (14 days).
