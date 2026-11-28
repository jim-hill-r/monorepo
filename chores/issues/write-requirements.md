---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Evals roadmap / Phase 2: Cover every project"
order: 10
added: 2026-11-28
---

Write chores's `docs/requirements.md` and `evals.toml` in the format of [standards/docs/evals.md](../../standards/docs/evals.md), for the routine-work commands: each command's output passes the audit the first time and the commands are deterministic. Draft must, should and could requirements from the README, plans and decisions, give every must at least one production-stage eval (a `cd` check of real content, or a `trycmd` case against the release binary for a CLI), and open the PR for Jim, who approves requirements. Today chores has none, so it adds nothing to the eval coverage figure. Part of the [evals roadmap](../../evals/docs/roadmap.md).
