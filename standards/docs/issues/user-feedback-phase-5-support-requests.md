---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "User feedback / Phase 5: support requests"
order: 50
added: 2026-11-28
---

Add support requests (option A in [the plan](../plans/2026-11-user-feedback.md#support-options)): `SupportForm` and `SupportThread`, `/api/support` in feedback/worker storing the request and a hash of a random 128-bit secret, the private link `/support/<id>#<secret>` kept in localStorage, replies read only when the user opens Help, closing after 30 days without a user reply and deletion 12 months after closing, and `feedback/manage`'s Feedback and Support inbox pages for decision 0018's management app component. Covers FDBK-010.
