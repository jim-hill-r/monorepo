# Priority Issues

## ISO 8601 Week Implementation Epic

This epic implements ISO 8601 week numbering (weeks 1-53, starting Monday, week 1 contains first Thursday) to replace the current simple day/7 calculation (weeks 1-52, no specific start day).

### Phase 1: Core Week Calculation
- TODO (agent-generated): Update `get_current_week_of_year()` to use ISO 8601 week calculation (chrono::Datelike::iso_week())
- TODO (agent-generated): Update `get_week_number_from_date()` to use ISO 8601 week calculation
- TODO (agent-generated): Change all week validation from `(1..=52)` to `(1..=53)` to support years with 53 weeks
- TODO (agent-generated): Update `get_week_start_date()` to return Monday (ISO 8601 start) instead of Sunday

### Phase 2: Recipe and Plan Display Logic
- TODO (agent-generated): Update `get_week_recipes()` to map ISO week numbers to correct day-of-year ranges
- TODO (agent-generated): Update `get_sidebar_recipe_days()` to use ISO week-based recipe retrieval
- TODO (agent-generated): Update `get_sidebar_plan_weeks()` to use ISO week numbering
- TODO (agent-generated): Update `get_week_shopping_list()` to work with ISO week numbers

### Phase 3: UI and Routing
- TODO (agent-generated): Update Route::Plan validation to accept weeks 1-53
- TODO (agent-generated): Update Plan component to handle weeks 1-53
- TODO (agent-generated): Update all UI text referring to "52 weeks" to "up to 53 weeks"

### Phase 4: Tests
- TODO (agent-generated): Update all week-related tests to validate ISO 8601 behavior
- TODO (agent-generated): Add tests for year boundaries (week 53 -> week 1 transitions)
- TODO (agent-generated): Add tests verifying Monday as week start day
- TODO (agent-generated): Add tests for years with 53 weeks (e.g., 2020, 2026)

### Phase 5: Documentation
- TODO (agent-generated): Update function documentation to describe ISO 8601 week numbering
- TODO (agent-generated): Add comments explaining ISO 8601 rules (week 1 = first week with Thursday)
- TODO (agent-generated): Update README with ISO 8601 week explanation

### Dependencies
- This epic must be completed before implementing the Plan data structure (next TODO)
- Recipe UUID refactoring (4th TODO) can be done independently

- TODO: Add plans to `cookbook/core` that stores the 7 recipes that are included in that week's plan.
- TODO: Store plans in the `content` directory similar to the recipes.
- TODO: Refactor recipes to include a UUID in the frontmatter of the markdown and also name the recipe files with the UUID. Ensure that the plans now reference the UUID for the recipe rather than the day number. The intent here is to allow future capability to rearrange what recipes belong to plans (ie decoupling a specific recipe from the day it is intended to be used on)

# Backlog

# Priority Projects
- web