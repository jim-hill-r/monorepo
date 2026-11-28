---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Cost tracking / Phase 4: Squarespace domains"
order: 40
added: 2026-11-27
---

Once Jim has answered `cost/docs/issues/human/list-squarespace-domains.md`, record each Squarespace domain (and any other paid Squarespace item) as a `[[cost]]` with `vendor = "Squarespace"`, `every = "year"`, its price in cents, `since` and `renews` (the next charge), in the `docs/cost.toml` of the project it serves, or in `standards/docs/cost.toml` when it serves none. Write one decision record in `standards/docs/decisions/` covering them all (what each costs a month and a year, why it's kept, how to cancel), set Squarespace's `budget` in `standards/radar/radar.json` to the number Jim gave, and add the first projects as Squarespace exemplars. Label the PR `needs-human`: COST-004 and COST-006 pass only once Jim accepts the record. Delete the human issue in the same PR.
