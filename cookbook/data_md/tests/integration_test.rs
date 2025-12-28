use cookbook_core::RecipeReader;
use cookbook_data_md::MarkdownRecipeStore;

#[test]
fn test_load_example_recipes() {
    let content_dir = "../content";
    
    let store = MarkdownRecipeStore::new(content_dir).expect("Should load content directory");
    
    // Test carbonara recipe exists and has data
    let carbonara = store.get_by_id("carbonara").expect("Should find carbonara recipe");
    assert_eq!(carbonara.id, "carbonara");
    assert_eq!(carbonara.title, "Spaghetti Carbonara");
    assert!(!carbonara.ingredients.is_empty(), "Should have ingredients");
    assert!(!carbonara.instructions.is_empty(), "Should have instructions");
    assert_eq!(carbonara.servings, Some(4));
    assert!(carbonara.has_tag("italian"));
    
    // Test day-1 recipe via get_by_day
    let day_1 = store.get_by_day(1).expect("Should find day-1 recipe");
    assert_eq!(day_1.id, "day-1");
    assert!(day_1.title.contains("Pancakes"));
    assert!(!day_1.ingredients.is_empty(), "Should have ingredients");
    assert!(!day_1.instructions.is_empty(), "Should have instructions");
    
    // Test get_all includes both recipes
    let all_recipes = store.get_all().expect("Should get all recipes");
    assert!(all_recipes.len() >= 2, "Should have at least 2 recipes");
    
    // Check that intro.md was not loaded as a recipe
    assert!(!store.exists("intro"), "intro.md should not be loaded as recipe");
}
