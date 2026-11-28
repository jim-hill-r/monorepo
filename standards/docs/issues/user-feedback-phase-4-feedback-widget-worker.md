---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "User feedback / Phase 4: feedback widget and worker"
order: 40
added: 2026-11-28
---

Add `feedback/widget` (form state, exact preview, one send per 30 seconds, same-origin fetch), the Dioxus `FeedbackForm`, and `feedback/worker`'s `/api/feedback` handler (validate with feedback/model, the product's own rate-limit binding, drop synthetic traffic, write through tenant_store). Playwright tests prove FDBK-001 to FDBK-004 and FDBK-008. See [the plan](../plans/2026-11-user-feedback.md#how-it-is-built).
