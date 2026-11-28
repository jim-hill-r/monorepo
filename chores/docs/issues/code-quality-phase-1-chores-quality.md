---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Code quality / Phase 1: Standard and ratchet"
order: 20
added: 2026-11-27
---

Add `chores quality` (`--check`, `--update`, `--report`): run clippy with `CLIPPY_CONF_DIR=standards/quality` (pedantic, cognitive_complexity and the thresholds in the plan), `--all-targets --message-format=json`, add the 800-line file check (skip generated code such as `infra/src/tfschema/`), and compare counts per lint with the project's `docs/quality.toml`. Fail only when a count rises, naming the new findings. Unit tests over fixture clippy JSON. Needs the `chores` crate (PR 863). Plan: [standards/docs/plans/2026-11-code-quality.md](../../../standards/docs/plans/2026-11-code-quality.md).
