---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Free tiers
order: 10
added: 2026-11-26
---

Once the nightly run deploys every project (PR #583), make `nightly-ci.yml` call `all-free-tier-usage.yml` as its first job and make the `mise-projects.yml` job need it, so a nightly deploy never runs past a budget.
