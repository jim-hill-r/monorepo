---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Resource cleanup / Phase 3: approved deletion"
order: 20
added: 2026-11-26
---

Resource cleanup phase 3 (see the [plan](../standards/docs/plans/2026-11-resource-cleanup.md)). The `jimhillr-{project}` rename has merged, so as soon as `standards/resources.toml` exists, add `[[retired]]` entries for `cahokia-web`, `pane` (`pane-dbc.pages.dev`) and `controlled-chaos` (`controlled-chaos-5v4.pages.dev`) in a `needs-human` PR. Jim deleted all three by hand on 2026-11-28 (the Pages API no longer lists them), so add them with `deleted = "2026-11-28"` and delete nothing. Their old addresses still appear in threat and workflow test fixtures, so those need `match_name = false` or new fixture names for RES-002 to pass.
