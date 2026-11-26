---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Review dashboard epic / Phase 7: revenue and cost"
order: 310
added: 2026-11-26
---

Add money types to `pane/model` (`Money` in cents, `RevenueLine`, `CostLine`, `ProjectMoney`) and pure functions for a project's monthly revenue, direct cost (spreading yearly costs over twelve months) and net, and for splitting shared costs across projects by their share of PRs merged that month, with unit tests for each edge (no PRs, one project, a PR touching two projects, a once cost).
