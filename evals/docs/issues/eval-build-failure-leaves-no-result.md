---
kind: fix
priority: priority
tags: [agent-generated]
epic: Eval engine
order: 90
added: 2026-11-27
---

A build that fails before its tests run leaves no eval result. Found by the [fire drill](../fire-drill-2026-11-27.md): removing cookbook's `day-200` tag stops `cookbook/data_md`'s build script, so `cargo test` prints no `test result:` lines, and `evals run --partial` (what CI uses for projects with sub-projects) writes 0 results; without `--partial` every `cargo` eval is `error` ("no test in the cargo report matched"), not `fail`. The PR check still fails, so nothing broken ships, but the requirement's verdict never shows the break. In `evals/cli/src/reports.rs`, when the saved `cargo test` output shows a build failure (`error: could not compile` or `failed to run custom build command`) and no test results, give each `cargo` eval of that report a `fail` result with the note "build failed: <first error line>", in `--partial` mode too (the report is that sub-project's own), and add a test with a saved build-failure log. Then rerun the drill's CI break.
