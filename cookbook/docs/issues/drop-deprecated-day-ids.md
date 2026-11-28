---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-28
---

Remove the deprecated `Recipe.id` (`day-N`) and `Plan.recipe_days` from `core`, `data_md` and the content files, and index `EmbeddedRecipeStore` by UUID instead of scanning all recipes. Plans already map to dates by ISO week. See the [deep review](../../../docs/reviews/2026-11-25-cookbook/deck.md), "Architecture findings".
