---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Resource cleanup / Phase 2: temporary resources"
order: 20
added: 2026-11-26
---

Resource cleanup phase 2 (see the [plan](../plans/2026-11-resource-cleanup.md)). Add `all-retire-resources.yml` (weekly after the sweep, and `workflow_dispatch` with `dry_run` defaulting to true) handling temporary resources only: delete Pages preview deployments older than 14 days (keeping the newest per branch with an open PR) and non-current production deployments older than 30 days, and delete branches 7 days after their PR closed. Move the branch-deletion logic from `all-delete-branches.yml` into `resources.sh` so both workflows share it. Branches with no PR are only listed in the sweep report.
