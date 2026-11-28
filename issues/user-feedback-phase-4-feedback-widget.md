---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "User feedback / Phase 4: feedback widget"
order: 40
added: 2026-11-28
---

Add `feedback/widget` (framework-free form state, exact preview text, one send per 30 seconds, `web-sys` fetch to a same-origin path the app gives) and the Dioxus `FeedbackForm`, with unit tests and Playwright tests proving FDBK-001 to FDBK-004 and FDBK-008: no request until Send, body equals preview, nothing but the listed fields. See [the plan](../standards/docs/plans/2026-11-user-feedback.md#how-it-is-built).
