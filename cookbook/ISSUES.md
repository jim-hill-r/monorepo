# Priority Issues

- TODO (agent-generated): Stop inventing UUIDs for recipes that lack one. Both `parse_recipe_markdown` functions in `data_md` fall back to `Uuid::new_v4()` when the `UUID:` line is missing or invalid, so such a recipe gets a new UUID on every load and plan references to it silently break. Return `RecipeError::InvalidData` instead, and make `MarkdownRecipeStore::load_recipes` and `load_plans` report files that fail to parse instead of dropping them with `if let Ok(..)`. Add tests for a missing and a malformed `UUID:` line.
- TODO (agent-generated): Remove panics from library code. `Plan::new` and `Plan::new_with_days` in `core/src/lib.rs` `assert_eq!` on the recipe count, and `EmbeddedRecipeStore::load_embedded_recipes` in `data_md/src/embedded.rs` `assert_eq!`s that 365 recipes loaded, which crashes the web app at startup. Make `Plan::new` return `PlanResult<Plan>` (folding in `new_checked`), and move the 365-recipe check into `data_md/build.rs` so a missing recipe fails the build instead of the running app.
- TODO (agent-generated): Make recipe IDs survive a save and reload in `MarkdownRecipeStore` (`data_md/src/lib.rs`). `write_recipe_file` names the file `{uuid}.md`, but `parse_recipe_markdown` uses the file stem as `id`, so a recipe created with id `day-1` comes back with its UUID as its id after a reload, and `create`/`update`/`exists` then miss it. Also `exists` ignores UUIDs although `get_by_id` accepts them, and `get_by_uuid` scans every recipe. Key the map by UUID with a legacy-id index, make `exists` match `get_by_id`, and add a create, reload, `get_by_id` round-trip test.

# Backlog

# Priority Projects
- web