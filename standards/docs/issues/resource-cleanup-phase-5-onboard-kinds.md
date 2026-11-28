---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Resource cleanup / Phase 5: everything onboarded to IaC"
order: 50
added: 2026-11-26
---

Resource cleanup phase 5 (see the [plan](../plans/2026-11-resource-cleanup.md), Part 6). As the IaC work (decision 0007) onboards each kind, add it to `[iac] managed` in the same PR so unmanaged becomes an error for it. Done when every kind in the plan's Part 6 table is managed or exempt with a reason. Each kind waits on its IaC stack.
