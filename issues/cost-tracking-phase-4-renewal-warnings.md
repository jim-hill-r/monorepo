---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Cost tracking / Phase 4: Squarespace domains"
order: 50
added: 2026-11-27
---

Add an optional `renews` date (`YYYY-MM-DD`, the next charge) to `[[cost]]` in `cost/model` and `standards/docs/cost.md`, required for `every = "year"` costs. Add a warning rule to `cost check` for a `renews` date in the past (the file wasn't updated after a renewal), a `cost renewals --within 60` command, and a line in the `human-issues-digest` skill's Cost section that lists renewals due in the next 60 days with their price, plus any domain `[[cost]]` whose project no longer deploys to it, as one to consider dropping. Unit tests for each.
