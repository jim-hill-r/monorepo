---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Review dashboard epic / Phase 5: actions"
added: 2026-11-27
---

Keep the Acknowledge page's acknowledgements across Jim's devices. Today
they live in one browser's `localStorage` (`pane.acks.v1`), so his phone
and laptop each start empty. Once the pane Worker and its write path exist,
store them there (KV, or a file committed by the `pane-edit` PR) and read
them back on every device ([decision 0014](../../../standards/docs/decisions/0014-faster-pr-approvals.md)).
