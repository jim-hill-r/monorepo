# Priority Issues

## ISO 8601 Week Implementation Epic

This epic implements ISO 8601 week numbering (weeks 1-53, starting Monday, week 1 contains first Thursday) to replace the current simple day/7 calculation (weeks 1-52, no specific start day).

### Phase 1: Core Week Calculation ✅ COMPLETED
All Phase 1 items have been completed:
- ✅ Updated `get_week_number_from_date()` to use ISO 8601 week calculation
- ✅ Changed all week validation from `(1..=52)` to `(1..=53)` to support years with 53 weeks
- ✅ Updated `get_week_start_date()` to return Monday (ISO 8601 start) instead of Sunday
- ✅ Updated Route::Plan validation to accept weeks 1-53
- ✅ Updated Plan component to handle weeks 1-53
- ✅ Updated all UI text referring to "52 weeks" to "up to 53 weeks"
- ✅ Updated all week-related tests to validate ISO 8601 behavior
- ✅ Added tests for year boundaries and years with 53 weeks (2020, 2026)
- ✅ Added tests verifying Monday as week start day
- ✅ Updated function documentation to describe ISO 8601 week numbering
- ✅ Added comments explaining ISO 8601 rules

### Phase 2: Recipe and Plan Display Logic ✅ COMPLETED
All Phase 2 items have been completed:
- ✅ Updated `get_week_recipes()` to map ISO week numbers to correct day-of-year ranges
- ✅ Updated `get_sidebar_recipe_days()` to use ISO week-based recipe retrieval (calls `get_week_recipes()`)
- ✅ Updated `get_sidebar_plan_weeks()` to use ISO week numbering (already used `get_week_number_from_date()`)
- ✅ Updated `get_week_shopping_list()` to work with ISO week numbers
- ✅ Added helper function `get_day_of_year_for_iso_week()` to convert ISO week numbers to day-of-year values
- ✅ Updated tests to validate ISO week mapping behavior
- ✅ Updated function documentation to describe ISO 8601 week mapping

### Dependencies
- This epic must be completed before implementing the Plan data structure (next TODO)
- Recipe UUID refactoring (4th TODO) can be done independently

### Phase 3: Plan Data Structure Implementation

- ✅ Create `Plan` struct in `cookbook/core` with fields for week number and list of 7 day-of-year values (representing recipes for Mon-Sun).
- ✅ Add `PlanError` enum to `cookbook/core` for plan-related errors (NotFound, InvalidData, AlreadyExists, etc).
- ✅ Add `PlanReader` trait to `cookbook/core` with methods: `get_by_week(week: u32) -> PlanResult<Plan>`, `get_all() -> PlanResult<Vec<Plan>>`, `exists(week: u32) -> bool`.
- ✅ Add `PlanWriter` trait to `cookbook/core` with methods: `create(plan: Plan)`, `update(plan: Plan)`, `delete(week: u32)`, `save(plan: Plan)`.
- ✅ Add comprehensive tests for `Plan` struct, `PlanError`, `PlanReader`, and `PlanWriter` traits in `cookbook/core/src/lib.rs`.
- ✅ Create plan content files in `cookbook/content/` directory with format `week-1.md`, `week-2.md`, etc. containing week metadata and 7 day references.
- ✅ Update `cookbook/data_md` to parse plan markdown files and implement `PlanReader` trait for reading plans from embedded content.
- ✅ Add tests to `cookbook/data_md` for plan parsing and `PlanReader` implementation.
- ✅ Update `cookbook/web` to use `PlanReader` trait to load plans instead of dynamically calculating week recipes.
- ✅ Add tests to `cookbook/web` to verify plan-based recipe retrieval.

### Phase 4: Recipe UUID Refactoring

- TODO: Refactor recipes to include a UUID in the frontmatter of the markdown and also name the recipe files with the UUID. Ensure that the plans now reference the UUID for the recipe rather than the day number. The intent here is to allow future capability to rearrange what recipes belong to plans (ie decoupling a specific recipe from the day it is intended to be used on)

# Backlog

# Priority Projects
- web