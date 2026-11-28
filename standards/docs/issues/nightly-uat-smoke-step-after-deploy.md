---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Feedback on every change
order: 130
added: 2026-11-26
---

Nightly UAT. Add a `smoke` step after `deploy` in the nightly run (`nightly-cd.yml` through `mise-projects.yml`, plus a smoke-only pass for projects that don't deploy) for every project, including on-hold ones, and publish the combined UAT report to `uat-reports` under a `nightly` branch.
