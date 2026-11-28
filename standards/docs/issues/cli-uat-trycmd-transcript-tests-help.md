---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Feedback on every change
order: 80
added: 2026-11-26
---

CLI UAT. Add `trycmd` transcript tests (`--help`, one happy path per main command, one common error) to cookbook/recipe_gen and my_lang, and a `smoke` task in `standards/tasks/rust.toml` that runs them against the release binary. homer already has them (`homer/tests/cli.rs`, `homer/tests/cmd/`); copy its setup.
