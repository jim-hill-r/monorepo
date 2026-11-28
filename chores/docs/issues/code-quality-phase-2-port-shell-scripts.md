---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Code quality / Phase 2: Other languages"
order: 45
added: 2026-11-27
---

Port the tracked `.sh` scripts still listed in `standards/shell-scripts.toml` (55 on 2026-11-28, most in `standards/scripts/`) into `chores` commands, one or two per PR, starting with the ones the hooks and PR CI run most (`cost-check.sh` and `royalty-check.sh` next, the same shape as `chores evals-check`). Each port moves the script's tests into Rust unit tests for the new command, then deletes the script and its line in the list; QUAL-404 (`chores shell-scripts`, in the audit) fails a new `.sh` file or a stale line. Jim chose "Keep Rust-only, port over time" (2026-11-27). Plan: [standards/docs/plans/2026-11-code-quality.md](../../../standards/docs/plans/2026-11-code-quality.md).
