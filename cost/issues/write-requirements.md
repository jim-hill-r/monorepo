---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Evals roadmap / Phase 2: Cover every project"
order: 10
added: 2026-11-28
---

Write cost's `docs/requirements.md` and `evals.toml` in the format of [standards/docs/evals.md](../../standards/docs/evals.md), for the cost check: every project's cost.toml is checked against its budget and free limits, and its ledger rules (double entry, every cent attributed to one project, decision 0016). Draft must, should and could requirements from the README, plans and decisions, give every must at least one production-stage eval (a `cd` check of real content, or a `trycmd` case against the release binary for a CLI), and open the PR for Jim, who approves requirements. Today cost has none, so it adds nothing to the eval coverage figure. Part of the [evals roadmap](../../evals/docs/roadmap.md).
