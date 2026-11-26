---
kind: todo
priority: priority
tags: [agent-generated]
added: 2026-11-26
---

Make the shopping list add up: `get_week_shopping_list` in `web/src/main.rs` sorts and `dedup`s identical strings, so week 48 shows 16 lines for 42 needed and lists olive oil once instead of 7 times. Parse quantity, unit and item, sum per item, list staples (oil, salt and pepper, garlic) separately, and test it. See the [deep review 2026-11-25](../../docs/reviews/2026-11-25-cookbook/deck.md).
