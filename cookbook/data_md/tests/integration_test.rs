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
    assert_eq!(plan_1.recipe_days.len(), 7);
    assert_eq!(plan_1.recipe_days, vec![1, 2, 3, 4, 5, 6, 7]);

    // Test get_all plans
    let all_plans = PlanReader::get_all(&store).expect("Should get all plans");
    assert!(
        all_plans.len() >= 52,
        "Should have at least 52 plans, got {}",
        all_plans.len()
    );
}
