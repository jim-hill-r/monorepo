---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Royalty / Phase 3: metered"
order: 10
added: 2026-11-27
---

Add `royalty count build` and `royalty count release`: one workflow step that appends a line (period, project, lockfile hash) to a counts file in the consumer's repo, and the `build` and `release` units read from it. No change to the consumer's program.
