---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Code quality / Phase 1: Standard and ratchet"
order: 30
added: 2026-11-27
---

Commit `docs/quality.toml` baselines for every project (`chores quality --update`); add `--all-targets`, `allow-unwrap-in-tests`, `allow-expect-in-tests` and the hard lints `dbg_macro`, `todo`, `allow_attributes_without_reason` to `standards/tasks/rust.toml` and every Cargo.toml; run `chores quality --check` in the hooks and a PR CI **Quality** job that posts changed counts; add the audit check that each project has a baseline or graduated hard lints (QUAL-502), and that cahokia's `clippy.toml` copies the shared thresholds. Plan: [standards/docs/plans/2026-11-code-quality.md](../standards/docs/plans/2026-11-code-quality.md). `chores quality` landed with the code quality phase 1 command PR; its first `--report` (2026-11-28) found 1,089 findings in 23 projects, and could not build cahokia, controlled_chaos, luggage, my_lang and starcraft in a cloud sandbox (system libraries such as `gdk-3.0`), so their baselines need the libraries CI installs (`ci:deps`).
