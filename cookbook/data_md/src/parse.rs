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
//! Plan files carry a `Recipe UUIDs: a, b, c, d, e, f, g` line.

use cookbook_core::{Plan, PlanError, PlanResult, Recipe};
use uuid::Uuid;

/// Builds a recipe from markdown `content`.
///
/// `id` is the recipe's legacy ID, also used as the title when the content has no `# ` heading.
pub(crate) fn recipe(content: &str, id: String, fallback_title: String) -> Recipe {
    let title = title(content).unwrap_or(fallback_title);

    // TODO (agent-generated): see cookbook/ISSUES.md, a missing UUID should be an error.
    let uuid = uuid(content).unwrap_or_else(Uuid::new_v4);

    let mut recipe = Recipe::new_with_uuid(id, uuid, title);
    recipe.description = description(content);
    recipe.ingredients = list_section(content, "## Ingredients");
    recipe.instructions = list_section(content, "## Instructions");
    recipe.prep_time_minutes = number_field(content, "Prep Time:");
    recipe.cook_time_minutes = number_field(content, "Cook Time:");
    recipe.servings = number_field(content, "Servings:");
    recipe.tags = tags(content);
    recipe
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

/// Non-empty lines between the title and the first `## ` section, joined with spaces.
// TODO (agent-generated): this also picks up the `UUID:`, `Prep Time:`, `Cook Time:`,
// `Servings:` and `Tags:` lines, which sit above `## Ingredients` in every recipe file.
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

        if in_description && !trimmed.is_empty() {
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

/// The value of the first `UUID:` line, or `None` if it is missing or invalid.
fn uuid(content: &str) -> Option<Uuid> {
    let value = content
        .lines()
        .find_map(|line| line.strip_prefix("UUID:"))?;
    Uuid::parse_str(value.trim()).ok()
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
        let recipe = recipe(RECIPE, "day-1".to_string(), "Day 1".to_string());

        assert_eq!(recipe.id, "day-1");
        assert_eq!(recipe.uuid.to_string(), UUID_A);
        assert_eq!(recipe.title, "Pancakes");
        assert!(
            recipe
                .description
                .as_deref()
                .is_some_and(|d| d.starts_with("Fluffy pancakes. Good for breakfast."))
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
        let recipe = recipe("no heading here", "x".to_string(), "Fallback".to_string());

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
    fn uuid_returns_none_when_missing_or_invalid() {
        assert_eq!(uuid("# Title"), None);
        assert_eq!(uuid("UUID: not-a-uuid"), None);
        assert_eq!(
            uuid(&format!("UUID:  {}  ", UUID_A)).map(|u| u.to_string()),
            Some(UUID_A.to_string())
        );
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
