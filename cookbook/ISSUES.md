# Priority Issues

- TODO (agent-generated): Remove panics from library code. `Plan::new` and `Plan::new_with_days` in `core/src/lib.rs` `assert_eq!` on the recipe count, and `EmbeddedRecipeStore::load_embedded_recipes` in `data_md/src/embedded.rs` `assert_eq!`s that 365 recipes loaded, which crashes the web app at startup. Make `Plan::new` return `PlanResult<Plan>` (folding in `new_checked`), and move the 365-recipe check into `data_md/build.rs` so a missing recipe fails the build instead of the running app.
- TODO (agent-generated): Fix `content/week-53.md`. It lists only 2 recipe UUIDs (days 365 and 1), so the plan never loads and `MarkdownRecipeStore::load_errors` reports it. Decide whether week 53 should hold 7 recipes (days 364 to 5 of the next year, say) or be dropped, and update `data_md/tests/integration_test.rs`, which allows this one error.
- TODO (agent-generated): Give `EmbeddedRecipeStore` (`data_md/src/embedded.rs`) the same UUID-keyed map as `MarkdownRecipeStore`: its `exists` ignores UUIDs although `get_by_id` accepts them, and `get_by_uuid` scans every recipe.

# Backlog

# Priority Projects
- web