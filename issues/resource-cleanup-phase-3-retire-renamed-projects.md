---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Resource cleanup / Phase 3: approved deletion"
order: 20
added: 2026-11-26
---

Resource cleanup phase 3 (see the [plan](../standards/docs/plans/2026-11-resource-cleanup.md)). Once the `jimhillr-{project}` rename moves `cahokia-web`, `pane` (`pane-dbc.pages.dev`) and `controlled-chaos` (`controlled-chaos-5v4.pages.dev`) to new Pages projects, add their `[[retired]]` entries (in the rename PR, or a separate `needs-human` PR if the rename already merged), plus a human issue to remove cahokia's old Auth0 callback, origin and logout URLs. If the rename lands before phase 1, add the entries as soon as `standards/resources.toml` exists.
