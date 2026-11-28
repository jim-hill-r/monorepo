---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Resource cleanup / Phase 1: claims, guards and a read-only sweep"
order: 35
added: 2026-11-28
---

Resource cleanup phase 1 (see the [plan](../plans/2026-11-resource-cleanup.md)). The resource sweep (`resources.sh sweep`) and the live threat check (`threat-live.sh`) now run in the same `threat-live` job of `all-free-tier-usage.yml`, but each lists Pages projects and Workers on its own. Have the sweep also record each Pages project's domains and newest preview URL in `live.json`, and have `threat-live.sh` read that file instead of calling the Cloudflare API again, so there is one listing and one report. Drop the threat check's own orphan finding for Pages and Workers once the sweep's `orphan` status covers it, so an orphan isn't reported twice.
