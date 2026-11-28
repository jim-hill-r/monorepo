---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "User feedback / Phase 7: QuickVote"
order: 70
added: 2026-11-28
---

Add `QuickVote` to `feedback/dioxus` and move bear_down/web's vote buttons and note box onto it. Move the votes and exposures from the shared telemetry Worker to bear_down's own endpoint (feedback/worker) and its tenant tables, keeping the A/B arms, the weekly per-arm summary and the Playwright tests. See [the user feedback plan](../../../standards/docs/plans/2026-11-user-feedback.md#what-already-exists-and-how-it-fits).
