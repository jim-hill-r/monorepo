---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Evals roadmap / Phase 2: Cover every project"
order: 10
added: 2026-11-28
---

Write infra's `docs/requirements.md` and `evals.toml` in the format of [standards/docs/evals.md](../../standards/docs/evals.md), for the infrastructure code: plans never put a secret in state (IAC-002), applies of credential changes wait for the second approval (decision 0010), and drift is reported. Draft must, should and could requirements from the README, plans and decisions, give every must at least one production-stage eval (a `cd` check of real content, or a `trycmd` case against the release binary for a CLI), and open the PR for Jim, who approves requirements. Today infra has none, so it adds nothing to the eval coverage figure. Part of the [evals roadmap](../../evals/docs/roadmap.md).
