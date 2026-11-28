---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Resource cleanup / Phase 3: approved deletion"
order: 30
added: 2026-11-28
---

Resource cleanup phase 3 (see the [plan](../standards/docs/plans/2026-11-resource-cleanup.md)). `chores retire-approved` and `all-retire-resources.yml` delete only retired Pages projects, the one kind the deploy token may delete; every other kind is listed as "delete by hand". When the first retired Worker, D1 database, KV namespace or DNS record exists, extend them: export a D1 or KV dump to a workflow artifact kept 90 days before deleting it, add the delete permission to the deploy token in `infra/scripts/mint.sh` (a credential change Jim approves), and when a retired hostname has a Cloudflare Access application, remove it after the resource (step 4 of the safe order in `standards/docs/resources.md`). Until then there is nothing to delete, so nothing to build.
