---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Resource cleanup / Phase 5: everything onboarded to IaC"
order: 40
added: 2026-11-26
---

Resource cleanup phase 5 (see the [plan](../plans/2026-11-resource-cleanup.md), Part 6). Write the Cloudflare, Auth0 (once its token exists) and PostHog live settings the sweep reads to `standards/live/<area>.json` (sorted, secrets never included), committed through the digest's agent step, so every hand change shows up as a diff and the IaC work can start from the snapshot.
