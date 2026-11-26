# Priority Issues

- TODO (agent-generated): Fix `content/week-53.md`. It lists only 2 recipe UUIDs (days 365 and 1), so the plan never loads and `MarkdownRecipeStore::load_errors` reports it. Decide whether week 53 should hold 7 recipes (days 364 to 5 of the next year, say) or be dropped, and update `data_md/tests/integration_test.rs`, which allows this one error.
- TODO (agent-generated): Give `EmbeddedRecipeStore` (`data_md/src/embedded.rs`) the same UUID-keyed map as `MarkdownRecipeStore`: its `exists` ignores UUIDs although `get_by_id` accepts them, and `get_by_uuid` scans every recipe.

# Backlog

# Priority Projects
- web