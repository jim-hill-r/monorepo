---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Evals roadmap / Phase 2: Cover every project"
order: 70
added: 2026-11-26
---

The `trycmd` kind is collected (from the `Testing ... ok` lines in the saved `cargo test` output). Still to do: the `script` kind, reading the named checks of controlled_chaos's scripted run once it has one (`issues/bevy-uat-scripted-mode-controlled-chaos.md`); and `cd`-stage `trycmd` evals against the released binaries of homer, cookbook/recipe_gen and my_lang, once their `smoke` task (`issues/cli-uat-trycmd-transcript-tests-help.md`) writes its output to `target/evals-reports/cd/cargo-test.log`. Each of those projects first needs a `docs/requirements.md` (Jim approves) and its case files named `tests/cmd/<ID>-<what>.toml`.
