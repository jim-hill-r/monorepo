---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Eval engine
added: 2026-11-27
---

Give LANG-001, LANG-002 and LANG-003 a production-stage eval. my_lang has no CLI, release or deploy, so its `must` requirements are checked only in `ci` and `evals check` warns EVL-004 for them. Once the CLI exists (`cli-build-and-run-commands`), add `tests/cmd/LANG-00n-*.toml` transcript cases (a program that runs, a broken program's error, an `input` with `return` rejected) and list them in `evals.toml` as `trycmd` evals with `stages = ["ci", "cd"]`, the way `homer/evals.toml` does, so they run against the release binary.
