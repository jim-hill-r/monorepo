use cookbook_core::{PlanReader, RecipeReader};
use cookbook_data_md::MarkdownRecipeStore;

#[test]
fn test_load_example_recipes() {
    let content_dir = "../content";

    let store = MarkdownRecipeStore::new(content_dir).expect("Should load content directory");

    // Every recipe and plan file must load.
    assert!(
        store.load_errors().is_empty(),
        "Unexpected load errors: {:?}",
        store.load_errors()
    );

    // Test plan loading first to get UUID for day-1
    let plan_1 = store.get_by_week(1).expect("Should find plan for week 1");
    assert_eq!(plan_1.week, 1);
    assert_eq!(plan_1.recipe_uuids.len(), 7);

    // Get day-1 recipe via UUID from plan (day-1 is Monday, first item in week 1)
    let day_1_uuid = plan_1.recipe_uuids[0];
    let day_1 = RecipeReader::get_by_uuid(&store, &day_1_uuid).expect("Should find day-1 recipe");

    // After migration, IDs are UUIDs, not day-based
    assert!(
        !day_1.id.starts_with("day-"),
        "ID should be UUID, not day-based"
    );
    assert!(day_1.title.contains("Pancakes"));
    assert!(!day_1.ingredients.is_empty(), "Should have ingredients");
    assert!(!day_1.instructions.is_empty(), "Should have instructions");
    // Verify the recipe still has the day-1 tag
    assert!(
        day_1.tags.contains(&"day-1".to_string()),
        "Should have day-1 tag"
    );

    // Test get_all includes many recipes
    let all_recipes = RecipeReader::get_all(&store).expect("Should get all recipes");
    assert_eq!(all_recipes.len(), 365, "Should have one recipe per day");

    // Check that intro.md was not loaded as a recipe
    assert!(
        !RecipeReader::exists(&store, "intro"),
        "intro.md should not be loaded as recipe"
    );

    // Verify the UUIDs correspond to recipes for days 1-7
    for (i, uuid) in plan_1.recipe_uuids.iter().enumerate() {
        let recipe = RecipeReader::get_by_uuid(&store, uuid).expect("Recipe should exist");
        let expected_day = i + 1;
        // Find the day tag in the recipe
        let day_tag = format!("day-{}", expected_day);
        assert!(
            recipe.tags.contains(&day_tag),
            "Plan week 1 UUID at position {} should be for day {} (recipe has tags: {:?})",
            i,
            expected_day,
            recipe.tags
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

#[test]
fn test_every_week_plan_parses_with_seven_recipes() {
    let content_dir = "../content";
    let store = MarkdownRecipeStore::new(content_dir).expect("Should load content directory");

    let mut weeks: Vec<u32> = std::fs::read_dir(content_dir)
        .expect("Should read content directory")
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().into_string().ok()?;
            name.strip_prefix("week-")?
                .strip_suffix(".md")?
                .parse()
                .ok()
        })
        .collect();
    weeks.sort_unstable();

    // ISO 8601 years have 52 or 53 weeks, so every one of them needs a plan.
    assert_eq!(
        weeks,
        (1..=53).collect::<Vec<u32>>(),
        "One plan per ISO week"
    );

    for week in weeks {
        let plan = store
            .get_by_week(week)
            .unwrap_or_else(|e| panic!("week-{week}.md should parse: {e}"));
        assert_eq!(plan.week, week);
        assert_eq!(plan.recipe_uuids.len(), 7, "week-{week}.md needs 7 recipes");
        for uuid in &plan.recipe_uuids {
            RecipeReader::get_by_uuid(&store, uuid)
                .unwrap_or_else(|e| panic!("week-{week}.md lists unknown recipe {uuid}: {e}"));
        }
    }
}
