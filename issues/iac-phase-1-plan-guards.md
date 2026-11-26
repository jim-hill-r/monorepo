---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Infrastructure as code / Phase 1: crate, Auth0 and pipeline"
order: 30
added: 2026-11-26
---

Decision 0007, item 6: a Rust `infra guard <plan.json>` command over `tofu show -json` that fails on IAC-001 (any delete or replace), IAC-002 (any sensitive value set), IAC-003 (resources past a free-tier budget, read from the same budgets as `free-tiers.sh`, or anything that can bill such as R2) and IAC-004 (a provider or resource type not on the allowlist), with unit tests on fixture plans.
