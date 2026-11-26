//! Markdown parsing shared by [`crate::MarkdownRecipeStore`] and the embedded stores.
//!
//! Recipe files look like this:
//!
//! ```text
//! # Title
//!
//! Description text.
//!
//! UUID: 550e8400-e29b-41d4-a716-446655440000
//! ID: day-1
//! Prep Time: 10 minutes
//! Cook Time: 20 minutes
//! Servings: 4
//! Tags: quick, vegetarian
//!
//! ## Ingredients
//! - item
//!
//! ## Instructions
//! 1. step
//! ```
//!
//! The `UUID:` line is required. The `ID:` line is optional and holds the recipe's legacy ID
//! when it differs from the UUID; without it the caller's fallback ID is used.
//!
//! Plan files carry a `Recipe UUIDs: a, b, c, d, e, f, g` line.

use cookbook_core::{Plan, PlanError, PlanResult, Recipe, RecipeError, RecipeResult};
use uuid::Uuid;

/// Lines above `## Ingredients` that hold a field rather than description text.
const FIELD_PREFIXES: [&str; 6] = [
    "UUID:",
    "ID:",
    "Prep Time:",
    "Cook Time:",
    "Servings:",
    "Tags:",
];

/// Builds a recipe from markdown `content`.
///
/// The recipe's legacy ID comes from the `ID:` line, or is `fallback_id` when there is none.
/// `fallback_title` is used when the content has no `# ` heading.
///
/// Fails with [`RecipeError::InvalidData`] when the `UUID:` line is missing or invalid, so a
/// recipe never gets a different UUID each time it is loaded.
pub(crate) fn recipe(
    content: &str,
    fallback_id: String,
    fallback_title: String,
) -> RecipeResult<Recipe> {
    let uuid = uuid(content)?;
    let id = text_field(content, "ID:").unwrap_or(fallback_id);
    let title = title(content).unwrap_or(fallback_title);

    let mut recipe = Recipe::new_with_uuid(id, uuid, title);
    recipe.description = description(content);
    recipe.ingredients = list_section(content, "## Ingredients");
    recipe.instructions = list_section(content, "## Instructions");
    recipe.prep_time_minutes = number_field(content, "Prep Time:");
    recipe.cook_time_minutes = number_field(content, "Cook Time:");
    recipe.servings = number_field(content, "Servings:");
    recipe.tags = tags(content);
    Ok(recipe)
}

/// Builds the plan for `week` from markdown `content`, which must list exactly 7 recipe UUIDs.
pub(crate) fn plan(content: &str, week: u32) -> PlanResult<Plan> {
    let recipe_uuids = plan_recipe_uuids(content)?;

    if recipe_uuids.len() != 7 {
        return Err(PlanError::InvalidData(format!(
            "Plan for week {} must have exactly 7 recipe UUIDs, found {}",
            week,
            recipe_uuids.len()
        )));
    }

    Plan::new_checked(week, recipe_uuids).map_err(|e| {
        PlanError::InvalidData(format!("Failed to create plan for week {}: {}", week, e))
    })
}

/// The first `# ` heading.
fn title(content: &str) -> Option<String> {
    content
        .lines()
        .find_map(|line| line.trim().strip_prefix("# "))
        .map(|title| title.trim().to_string())
}

/// Non-empty lines between the title and the first `## ` section, joined with spaces,
/// leaving out field lines such as `UUID:` and `Tags:`.
fn description(content: &str) -> Option<String> {
    let mut desc_lines = Vec::new();
    let mut in_description = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("# ") {
            in_description = true;
            continue;
        }

        if trimmed.starts_with("## ") {
            break;
        }

        let is_field = FIELD_PREFIXES
            .iter()
            .any(|prefix| trimmed.starts_with(prefix));

        if in_description && !trimmed.is_empty() && !is_field {
            desc_lines.push(trimmed.to_string());
        }
    }

    if desc_lines.is_empty() {
        None
    } else {
        Some(desc_lines.join(" "))
    }
}

/// `- ` and `1. ` list items under `section_header`, up to the next `## ` section.
fn list_section(content: &str, section_header: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut in_section = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == section_header {
            in_section = true;
            continue;
        }

        if in_section && trimmed.starts_with("## ") {
            break;
        }

        if in_section {
            if let Some(item) = trimmed.strip_prefix("- ") {
                items.push(item.trim().to_string());
            } else if let Some(pos) = trimmed.find(". ")
                && trimmed[..pos].chars().all(|c| c.is_ascii_digit())
            {
                items.push(trimmed[pos + 2..].trim().to_string());
            }
        }
    }

    items
}

/// The leading number of the first line starting with `field`, as in `Prep Time: 10 minutes`.
fn number_field(content: &str, field: &str) -> Option<u32> {
    let value = content.lines().find_map(|line| line.strip_prefix(field))?;
    value.split_whitespace().next()?.parse::<u32>().ok()
}

/// Comma-separated values of the first `Tags:` line.
fn tags(content: &str) -> Vec<String> {
    content
        .lines()
        .find_map(|line| line.strip_prefix("Tags:"))
        .map(|value| {
            value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// The trimmed value of the first line starting with `field`, or `None` if it is missing or empty.
fn text_field(content: &str, field: &str) -> Option<String> {
    let value = content
        .lines()
        .find_map(|line| line.strip_prefix(field))?
        .trim();
    (!value.is_empty()).then(|| value.to_string())
}

/// The value of the first `UUID:` line.
fn uuid(content: &str) -> RecipeResult<Uuid> {
    let value = text_field(content, "UUID:")
        .ok_or_else(|| RecipeError::InvalidData("Missing 'UUID:' line".to_string()))?;
    Uuid::parse_str(&value)
        .map_err(|e| RecipeError::InvalidData(format!("Invalid UUID '{}': {}", value, e)))
}

/// The UUIDs on the `Recipe UUIDs:` line.
fn plan_recipe_uuids(content: &str) -> PlanResult<Vec<Uuid>> {
    let uuids_str = content
        .lines()
        .find_map(|line| line.trim().strip_prefix("Recipe UUIDs:"))
        .ok_or_else(|| {
            PlanError::InvalidData(
                "Could not find 'Recipe UUIDs:' line in plan markdown".to_string(),
            )
        })?;

    uuids_str
        .split(',')
        .map(|s| {
            Uuid::parse_str(s.trim())
                .map_err(|e| PlanError::InvalidData(format!("Invalid UUID: {}", e)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const UUID_A: &str = "550e8400-e29b-41d4-a716-446655440001";

    const RECIPE: &str = "# Pancakes

Fluffy pancakes.
Good for breakfast.

UUID: 550e8400-e29b-41d4-a716-446655440001
Prep Time: 10 minutes
Cook Time: 15
Servings: 4 people
Tags: breakfast, , sweet

## Ingredients
- 2 cups flour
- 1 egg

## Instructions
1. Mix.
2. Cook.
Not a list item
10. Serve.

## Notes
- Not an ingredient
";

    #[test]
    fn recipe_parses_every_field() {
        let recipe = recipe(RECIPE, "day-1".to_string(), "Day 1".to_string()).unwrap();

        assert_eq!(recipe.id, "day-1");
        assert_eq!(recipe.uuid.to_string(), UUID_A);
        assert_eq!(recipe.title, "Pancakes");
        assert_eq!(
            recipe.description.as_deref(),
            Some("Fluffy pancakes. Good for breakfast.")
        );
        assert_eq!(recipe.ingredients, vec!["2 cups flour", "1 egg"]);
        assert_eq!(recipe.instructions, vec!["Mix.", "Cook.", "Serve."]);
        assert_eq!(recipe.prep_time_minutes, Some(10));
        assert_eq!(recipe.cook_time_minutes, Some(15));
        assert_eq!(recipe.servings, Some(4));
        assert_eq!(recipe.tags, vec!["breakfast", "sweet"]);
    }

    #[test]
    fn recipe_uses_fallback_title_and_defaults_when_fields_missing() {
        let recipe = recipe(
            &format!("UUID: {}", UUID_A),
            "x".to_string(),
            "Fallback".to_string(),
        )
        .unwrap();

        assert_eq!(recipe.id, "x");
        assert_eq!(recipe.title, "Fallback");
        assert_eq!(recipe.description, None);
        assert!(recipe.ingredients.is_empty());
        assert!(recipe.instructions.is_empty());
        assert_eq!(recipe.prep_time_minutes, None);
        assert_eq!(recipe.cook_time_minutes, None);
        assert_eq!(recipe.servings, None);
        assert!(recipe.tags.is_empty());
    }

    #[test]
    fn number_field_rejects_non_numbers() {
        assert_eq!(number_field("Servings: many", "Servings:"), None);
        assert_eq!(number_field("Servings:", "Servings:"), None);
    }

    #[test]
    fn recipe_takes_id_from_id_line() {
        let content = format!("# Soup\n\nUUID: {}\nID: day-7\n", UUID_A);
        let recipe = recipe(&content, "file-stem".to_string(), "Soup".to_string()).unwrap();

        assert_eq!(recipe.id, "day-7");
        assert_eq!(recipe.description, None);
    }

    #[test]
    fn recipe_rejects_missing_uuid() {
        assert!(matches!(
            recipe("# Title\n\nText.\n", "x".to_string(), "X".to_string()),
            Err(RecipeError::InvalidData(_))
        ));
    }

    #[test]
    fn recipe_rejects_malformed_uuid() {
        assert!(matches!(
            recipe(
                "# Title\n\nUUID: not-a-uuid\n",
                "x".to_string(),
                "X".to_string()
            ),
            Err(RecipeError::InvalidData(_))
        ));
    }

    #[test]
    fn uuid_errors_when_missing_or_invalid() {
        assert!(matches!(uuid("# Title"), Err(RecipeError::InvalidData(_))));
        assert!(matches!(uuid("UUID:   "), Err(RecipeError::InvalidData(_))));
        assert!(matches!(
            uuid("UUID: not-a-uuid"),
            Err(RecipeError::InvalidData(_))
        ));
        assert_eq!(
            uuid(&format!("UUID:  {}  ", UUID_A)).unwrap().to_string(),
            UUID_A
        );
    }

    #[test]
    fn description_skips_field_lines_in_any_order() {
        let content = format!(
            "# T\n\nUUID: {}\n\nFirst.\n\nPrep Time: 1\nSecond.\nTags: a\n\n## Ingredients\n",
            UUID_A
        );
        assert_eq!(description(&content).as_deref(), Some("First. Second."));
    }

    fn plan_content(count: usize) -> String {
        let uuids: Vec<String> = (1..=count)
            .map(|i| format!("550e8400-e29b-41d4-a716-4466554400{:02}", i))
            .collect();
        format!(
            "# Week 3 Plan\n\nWeek: 3\nRecipe UUIDs: {}\n",
            uuids.join(", ")
        )
    }

    #[test]
    fn plan_parses_seven_uuids() {
        let plan = plan(&plan_content(7), 3).unwrap();

        assert_eq!(plan.week, 3);
        assert_eq!(plan.recipe_uuids.len(), 7);
        assert_eq!(plan.recipe_uuids[0].to_string(), UUID_A);
    }

    #[test]
    fn plan_rejects_wrong_uuid_count() {
        assert!(matches!(
            plan(&plan_content(2), 3),
            Err(PlanError::InvalidData(_))
        ));
    }

    #[test]
    fn plan_rejects_missing_uuid_line() {
        assert!(matches!(
            plan("Week: 5\nNo Recipe UUIDs field here\n", 5),
            Err(PlanError::InvalidData(_))
        ));
    }

    #[test]
    fn plan_rejects_invalid_uuid() {
        assert!(matches!(
            plan_recipe_uuids("Recipe UUIDs: not-a-uuid, also-not-a-uuid\n"),
            Err(PlanError::InvalidData(_))
        ));
    }
}
