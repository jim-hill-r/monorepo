---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Resource cleanup / Phase 2: temporary resources"
order: 10
added: 2026-11-26
---

Resource cleanup phase 2 (see the [plan](../standards/docs/plans/2026-11-resource-cleanup.md)). Add `pull-request-closed-cleanup.yml` on `pull_request: closed` that deletes the Actions caches for `refs/pull/N/merge` (GitHub's documented cache clean-up pattern), with `actions: write` only.
