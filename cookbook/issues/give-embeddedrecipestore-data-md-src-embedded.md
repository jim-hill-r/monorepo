---
kind: todo
priority: priority
tags: [agent-generated]
added: 2026-11-26
---

Give `EmbeddedRecipeStore` (`data_md/src/embedded.rs`) the same UUID-keyed map as `MarkdownRecipeStore`: its `exists` ignores UUIDs although `get_by_id` accepts them, and `get_by_uuid` scans every recipe.
