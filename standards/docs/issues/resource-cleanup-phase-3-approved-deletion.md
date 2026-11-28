---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Resource cleanup / Phase 3: approved deletion"
order: 10
added: 2026-11-26
---

Resource cleanup phase 3 (see the [plan](../plans/2026-11-resource-cleanup.md)). Make `all-retire-resources.yml` delete `[[retired]]` entries past `delete_after` in the safe order (inbound references gone, then the resource, with a D1/KV dump artifact kept 90 days first, then any Cloudflare Access application, then verify). Hard stops per entry: the name is still claimed on `main`, a Pages project had a production deploy in the last 7 days, or an inbound reference remains (including an open human issue for its Auth0 URLs). Never delete anything only because it is unclaimed.
