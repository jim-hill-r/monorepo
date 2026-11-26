---
kind: todo
priority: priority
tags: [agent-generated]
added: 2026-11-26
---

Split `web/src/main.rs` (1,795 lines): move the date and plan logic (`get_week_recipes`, `get_week_shopping_list`, `get_sidebar_*`, week helpers) into `core` with its tests, and the components into modules. Replace tests that only assert constants, such as `test_recipe_valid_day_in_range`. See the [deep review 2026-11-25](../../docs/reviews/2026-11-25-cookbook/deck.md).
