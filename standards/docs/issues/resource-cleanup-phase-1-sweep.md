---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Resource cleanup / Phase 1: claims, guards and a read-only sweep"
order: 30
added: 2026-11-26
---

Resource cleanup phase 1 (see the [plan](../plans/2026-11-resource-cleanup.md)). Add `resources.sh sweep` as a read-only job in `all-free-tier-usage.yml`, reusing the `free-tiers.sh` Cloudflare helpers and token: list Pages projects and deployments, Workers, D1, KV, Queues, Cron Triggers, DNS records (once a zone exists), branches, releases and Actions caches; sort each into claimed, kept, retired pending, retired but still live, kept past review, or orphan; write a run summary table and a JSON report artifact (`retention-days: 7`); fail on an orphan, a failed deletion or an overdue keep. This is the orphan half of the threat check's weekly live check: the threat check's URL and access probes should run in the same job and read the same report.
