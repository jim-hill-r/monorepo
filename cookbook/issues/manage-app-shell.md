---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Management app / Phase 2: the app shell"
order: 20
added: 2026-11-28
---

Create `cookbook/manage`, a Dioxus web app deployed as the Pages project `jimhillr-cookbook-manage`, in the earthy theme from `design_system`. It signs in through `auth_sdk`, shows a plain "you can't use this app" page to anyone without `manage:cookbook`, and lists recipes and plans read-only with each recipe's status and counts toward the book, using `core` and `data_md`. Ship it with its user guide for the curators (`docs/user/README.md`), `docs/cost.toml`, `docs/threat.toml`, requirements and evals, and its entry in `standards/resources.toml`. Needs [manage-auth0-curator-role](manage-auth0-curator-role.md) and `auth_sdk`'s [web-provider-todo-panics](../../auth_sdk/issues/web-provider-todo-panics.md) (`is_authenticated` and `logout` are `todo!()`). See the [roadmap](../docs/roadmap.md).
