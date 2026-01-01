use cookbook_core::{PlanReader, RecipeReader};
use cookbook_data_md::MarkdownRecipeStore;

#[test]
fn test_load_example_recipes() {
    let content_dir = "../content";

    let store = MarkdownRecipeStore::new(content_dir).expect("Should load content directory");

    // Test day-1 recipe via get_by_day
    let day_1 = store.get_by_day(1).expect("Should find day-1 recipe");
    assert_eq!(day_1.id, "day-1");
    assert!(day_1.title.contains("Pancakes"));
    assert!(!day_1.ingredients.is_empty(), "Should have ingredients");
    assert!(!day_1.instructions.is_empty(), "Should have instructions");

    // Test get_all includes many recipes
    let all_recipes = RecipeReader::get_all(&store).expect("Should get all recipes");
    assert!(all_recipes.len() >= 365, "Should have at least 365 recipes");

    // Check that intro.md was not loaded as a recipe
    assert!(
        !RecipeReader::exists(&store, "intro"),
        "intro.md should not be loaded as recipe"
    );

    // Test plan loading
    let plan_1 = store.get_by_week(1).expect("Should find plan for week 1");
    assert_eq!(plan_1.week, 1);
    assert_eq!(plan_1.recipe_uuids.len(), 7);

    // Verify the UUIDs correspond to recipes for days 1-7
    for (i, uuid) in plan_1.recipe_uuids.iter().enumerate() {
        let recipe = RecipeReader::get_by_uuid(&store, uuid).expect("Recipe should exist");
        let expected_day = i + 1;
        let day_from_id = recipe
            .id
            .strip_prefix("day-")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap();
        assert_eq!(
            day_from_id, expected_day as u32,
            "Plan week 1 UUID at position {} should be for day {}",
            i, expected_day
        );
    }

    // Test get_all plans
    let all_plans = PlanReader::get_all(&store).expect("Should get all plans");
    assert!(
        all_plans.len() >= 52,
        "Should have at least 52 plans, got {}",
        all_plans.len()
    );
}
