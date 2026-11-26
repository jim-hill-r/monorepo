# Priority Issues

- TODO (agent-generated): Make the shopping list add up: `get_week_shopping_list` in `web/src/main.rs` sorts and `dedup`s identical strings, so week 48 shows 16 lines for 42 needed and lists olive oil once instead of 7 times. Parse quantity, unit and item, sum per item, list staples (oil, salt and pepper, garlic) separately, and test it. See the [deep review 2026-11-25](../docs/reviews/2026-11-25-cookbook/deck.md).
- TODO (agent-generated): Add a `_headers` file to the Pages build with the CSP from `web/src/main.rs` plus `frame-ancestors 'none'` and `Referrer-Policy: no-referrer`, since a `<meta>` CSP can't stop framing. See the [deep review 2026-11-25](../docs/reviews/2026-11-25-cookbook/deck.md).
- TODO (agent-generated): Add a `[profile.release]` to `cookbook/Cargo.toml` (`opt-level = "z"`, `lto = true`, `codegen-units = 1`, `strip = true`) and record the wasm size before and after (3,432,336 bytes on 2026-11-25). See the [deep review 2026-11-25](../docs/reviews/2026-11-25-cookbook/deck.md).
- TODO (agent-generated): Split `web/src/main.rs` (1,795 lines): move the date and plan logic (`get_week_recipes`, `get_week_shopping_list`, `get_sidebar_*`, week helpers) into `core` with its tests, and the components into modules. Replace tests that only assert constants, such as `test_recipe_valid_day_in_range`. See the [deep review 2026-11-25](../docs/reviews/2026-11-25-cookbook/deck.md).
- TODO (agent-generated): Mobile header: below 768 px move Home / Recipes / Plans into the sidebar so the header is one row, and add previous / next day links on the recipe page. See the [deep review 2026-11-25](../docs/reviews/2026-11-25-cookbook/deck.md).
- TODO (agent-generated): Give `EmbeddedRecipeStore` (`data_md/src/embedded.rs`) the same UUID-keyed map as `MarkdownRecipeStore`: its `exists` ignores UUIDs although `get_by_id` accepts them, and `get_by_uuid` scans every recipe.

# Backlog

# Priority Projects
- web