---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Simple dashboard / Phase 3: domain scorecards"
order: 20
added: 2026-11-28
---

Add `pane_model::scorecards`: one scorecard per domain, worked out from the snapshot so it is testable without a browser ([roadmap](../docs/roadmap.md)). Domains: the repo health domains in `standards/repo-health.toml` (Build and deploy, Quality, Threats, Review, Infrastructure, Data) plus Requirements (eval coverage and must verdicts), Money (cost against budget, from `Snapshot.costs`) and Work (open TODOs and FIXes, human issues by age). Each has a light (worst of its signals, grey when unknown), one headline number, and the projects that make it worse, worst first. Thresholds go in `pane/health.toml`. The domain list is a guess: confirm it with Jim in the PR.
