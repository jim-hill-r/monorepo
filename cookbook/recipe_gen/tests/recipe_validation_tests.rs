use cookbook_core::RecipeReader;
use cookbook_data_md::MarkdownRecipeStore;

#[test]
fn test_all_365_day_files_exist() {
    let content_dir = "../content";
    let store = MarkdownRecipeStore::new(content_dir)
        .unwrap_or_else(|e| panic!("Failed to create recipe store: {}", e));

    // Verify all 365 day files exist
    let mut missing_days = Vec::new();
    for day in 1..=365 {
        let result = store.get_by_day(day);
        if result.is_err() {
            missing_days.push(day);
        }
    }

    assert!(
        missing_days.is_empty(),
        "Missing day files: {:?}",
        missing_days
    );
}

#[test]
fn test_all_day_recipes_have_required_fields() {
    let content_dir = "../content";
    let store = MarkdownRecipeStore::new(content_dir)
        .unwrap_or_else(|e| panic!("Failed to create recipe store: {}", e));

    let mut invalid_recipes = Vec::new();

    for day in 1..=365 {
        if let Ok(recipe) = store.get_by_day(day) {
            // Verify required fields
            let mut issues = Vec::new();

            if recipe.id.is_empty() {
                issues.push("empty id");
            }
            if recipe.title.is_empty() {
                issues.push("empty title");
            }
            if recipe.ingredients.is_empty() {
                issues.push("no ingredients");
            }
            if recipe.instructions.is_empty() {
                issues.push("no instructions");
            }

            if !issues.is_empty() {
                invalid_recipes.push((day, issues));
            }
        }
    }

    assert!(
        invalid_recipes.is_empty(),
        "Invalid recipes found: {:?}",
        invalid_recipes
    );
}

#[test]
fn test_all_day_recipes_have_proper_tags() {
    let content_dir = "../content";
    let store = MarkdownRecipeStore::new(content_dir)
        .unwrap_or_else(|e| panic!("Failed to create recipe store: {}", e));

    let mut recipes_without_tags = Vec::new();

    for day in 1..=365 {
        if let Ok(recipe) = store.get_by_day(day) {
            if recipe.tags.is_empty() {
                recipes_without_tags.push(day);
            } else {
                // Verify the recipe has the day tag
                let day_tag = format!("day-{}", day);
                assert!(
                    recipe.tags.contains(&day_tag),
                    "Recipe for day {} is missing the {} tag",
                    day,
                    day_tag
                );
            }
        }
    }

    assert!(
        recipes_without_tags.is_empty(),
        "Recipes without tags: {:?}",
        recipes_without_tags
    );
}

#[test]
fn test_recipe_variety_across_categories() {
    let content_dir = "../content";
    let store = MarkdownRecipeStore::new(content_dir)
        .unwrap_or_else(|e| panic!("Failed to create recipe store: {}", e));

    let mut category_counts = std::collections::HashMap::new();

    for day in 1..=365 {
        if let Ok(recipe) = store.get_by_day(day) {
            for tag in &recipe.tags {
                let tag_lower = tag.to_lowercase();
                if [
                    "breakfast",
                    "lunch",
                    "dinner",
                    "dessert",
                    "snack",
                    "appetizer",
                    "soup",
                    "salad",
                    "beverage",
                ]
                .contains(&tag_lower.as_str())
                {
                    *category_counts.entry(tag_lower).or_insert(0) += 1;
                }
            }
        }
    }

    // Verify we have at least some variety (should have multiple categories)
    assert!(
        category_counts.len() >= 5,
        "Not enough category variety. Found: {:?}",
        category_counts
    );
}

#[test]
fn test_recipe_variety_across_cuisines() {
    let content_dir = "../content";
    let store = MarkdownRecipeStore::new(content_dir)
        .unwrap_or_else(|e| panic!("Failed to create recipe store: {}", e));

    let mut cuisine_counts = std::collections::HashMap::new();

    for day in 1..=365 {
        if let Ok(recipe) = store.get_by_day(day) {
            for tag in &recipe.tags {
                let tag_lower = tag.to_lowercase();
                if [
                    "american",
                    "italian",
                    "chinese",
                    "mexican",
                    "indian",
                    "japanese",
                    "french",
                    "thai",
                    "greek",
                    "spanish",
                    "middle-eastern",
                    "korean",
                    "vietnamese",
                    "mediterranean",
                    "caribbean",
                ]
                .contains(&tag_lower.as_str())
                {
                    *cuisine_counts.entry(tag_lower).or_insert(0) += 1;
                }
            }
        }
    }

    // Verify we have at least some variety (should have multiple cuisines)
    assert!(
        cuisine_counts.len() >= 5,
        "Not enough cuisine variety. Found: {:?}",
        cuisine_counts
    );
}

#[test]
fn test_recipe_ids_match_day_format() {
    let content_dir = "../content";
    let store = MarkdownRecipeStore::new(content_dir)
        .unwrap_or_else(|e| panic!("Failed to create recipe store: {}", e));

    for day in 1..=365 {
        if let Ok(recipe) = store.get_by_day(day) {
            let expected_id = format!("day-{}", day);
            assert_eq!(
                recipe.id, expected_id,
                "Recipe for day {} has incorrect id: {}",
                day, recipe.id
            );
        }
    }
}
