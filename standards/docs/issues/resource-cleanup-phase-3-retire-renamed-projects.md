---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Resource cleanup / Phase 3: approved deletion"
order: 20
added: 2026-11-26
---

Resource cleanup phase 3 (see the [plan](../plans/2026-11-resource-cleanup.md)). The `jimhillr-{project}` rename has merged, so as soon as `standards/resources.toml` exists, add `[[retired]]` entries for `cahokia-web`, `pane` (`pane-dbc.pages.dev`) and `controlled-chaos` (`controlled-chaos-5v4.pages.dev`) in a `needs-human` PR. Their deletion is already the human issue `delete-old-pages-projects-after-jimhillr-rename.md`. If Jim has done it, set `deleted` and delete nothing. Otherwise `all-retire-resources.yml` deletes them once phase 3 ships, and that PR deletes the human issue.
