---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "User feedback / Phase 8: one feedback tool with roles"
order: 85
added: 2026-11-28
---

Package the Feedback button's staff modes (user-feedback-phase-8-staff-modes.md, needed first) as a Manifest V3 browser extension that injects it into our production sites and sends from its background worker to pane, signed in through pane's Cloudflare Access login ([decision 0024](../standards/docs/decisions/0024-internal-sign-in.md): this is the only way product owner and engineering modes reach a product page), so production builds can carry no staff code and strict CSPs need no change. Release it unpacked from a GitHub release (no Chrome Web Store fee). See [the plan](../standards/docs/plans/2026-11-user-feedback.md#phases).
