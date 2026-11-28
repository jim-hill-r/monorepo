---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "User feedback / Phase 7: review toolbar extension"
order: 70
added: 2026-11-28
---

Package the eval engine's review toolbar (issues/eval-engine-phase-6-feedback-toolbar.md, needed first) as a Manifest V3 browser extension that injects it into our production sites and sends from its background worker, so production builds carry no review code and strict CSPs need no change. Release it unpacked from a GitHub release (no Chrome Web Store fee). See [the plan](../standards/docs/plans/2026-11-user-feedback.md#phases).
