---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Code quality / Phase 2: Other languages"
order: 45
added: 2026-11-27
---

Port the tracked `.sh` scripts (53 on 2026-11-27, most in `standards/scripts/`) into `chores` commands, one or two per PR, starting with the ones the hooks and PR CI run most. Each port keeps the script's tests passing against the new command, then deletes the script. Add the audit check (QUAL-404) that fails any new tracked `.sh` file, with the not-yet-ported scripts on an allow list that only shrinks. Jim chose "Keep Rust-only, port over time" (2026-11-27). Plan: [standards/docs/plans/2026-11-code-quality.md](../../../standards/docs/plans/2026-11-code-quality.md).
