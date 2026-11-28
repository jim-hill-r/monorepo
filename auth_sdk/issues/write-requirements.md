---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Evals roadmap / Phase 2: Cover every project"
order: 10
added: 2026-11-28
---

Write auth_sdk's `docs/requirements.md` and `evals.toml` in the format of [standards/docs/evals.md](../../standards/docs/evals.md), for the sign-in library cahokia and cookbook use: sign-in and sign-out flows, token handling and what it must never store or log. Draft must, should and could requirements from the README, plans and decisions, give every must at least one production-stage eval (a `cd` check of real content, or a `trycmd` case against the release binary for a CLI), and open the PR for Jim, who approves requirements. Today auth_sdk has none, so it adds nothing to the eval coverage figure. Part of the [evals roadmap](../../evals/docs/roadmap.md).
