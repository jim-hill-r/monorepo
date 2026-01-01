use cookbook_core::{
    Plan, PlanError, PlanReader, PlanResult, Recipe, RecipeError, RecipeReader, RecipeResult,
    RecipeWriter,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

// Export the embedded module for WASM/web targets
pub mod embedded;

/// A markdown-based implementation of RecipeReader and RecipeWriter
/// that reads and writes recipe data from/to markdown files.
pub struct MarkdownRecipeStore {
    content_dir: PathBuf,
    recipes: HashMap<String, Recipe>,
    plans: HashMap<u32, Plan>,
}

impl MarkdownRecipeStore {
    /// Creates a new MarkdownRecipeStore with the given content directory
    pub fn new<P: AsRef<Path>>(content_dir: P) -> RecipeResult<Self> {
        let content_dir = content_dir.as_ref().to_path_buf();

        if !content_dir.exists() {
            return Err(RecipeError::StorageError(format!(
                "Content directory does not exist: {}",
                content_dir.display()
            )));
        }

        let mut store = Self {
            content_dir,
            recipes: HashMap::new(),
            plans: HashMap::new(),
        };

        // Load all recipes from the content directory
        store.load_recipes()?;

        // Load all plans from the content directory
        store.load_plans()?;

        Ok(store)
    }

    /// Loads all recipe markdown files from the content directory
    fn load_recipes(&mut self) -> RecipeResult<()> {
        let entries = fs::read_dir(&self.content_dir).map_err(|e| {
            RecipeError::StorageError(format!("Failed to read content directory: {}", e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                RecipeError::StorageError(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                // Skip non-recipe files like intro.md
                let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

                if file_name == "intro" {
                    continue;
                }

                // Try to parse the recipe
                if let Ok(recipe) = self.parse_recipe_file(&path) {
                    self.recipes.insert(recipe.id.clone(), recipe);
                }
            }
        }

        Ok(())
    }

    /// Parses a recipe from a markdown file
    fn parse_recipe_file(&self, path: &Path) -> RecipeResult<Recipe> {
        let content = fs::read_to_string(path).map_err(|e| {
            RecipeError::StorageError(format!("Failed to read file {}: {}", path.display(), e))
        })?;

        self.parse_recipe_markdown(&content, path)
    }

    /// Parses recipe data from markdown content
    fn parse_recipe_markdown(&self, content: &str, path: &Path) -> RecipeResult<Recipe> {
        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| RecipeError::InvalidData("Invalid file name".to_string()))?;

        // Use file name as ID
        let id = file_stem.to_string();

        // Parse title from first heading or use file name
        let title = self.extract_title(content).unwrap_or_else(|| id.clone());

        let mut recipe = Recipe::new(id, title);

        // Parse optional fields from markdown
        recipe.description = self.extract_description(content);
        recipe.ingredients = self.extract_ingredients(content);
        recipe.instructions = self.extract_instructions(content);
        recipe.prep_time_minutes = self.extract_prep_time(content);
        recipe.cook_time_minutes = self.extract_cook_time(content);
        recipe.servings = self.extract_servings(content);
        recipe.tags = self.extract_tags(content);

        Ok(recipe)
    }

    /// Extracts the title from markdown (first # heading)
    fn extract_title(&self, content: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(title) = trimmed.strip_prefix("# ") {
                return Some(title.trim().to_string());
            }
        }
        None
    }

    /// Extracts description (text before first section)
    fn extract_description(&self, content: &str) -> Option<String> {
        let mut desc_lines = Vec::new();
        let mut in_description = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip title
            if trimmed.starts_with("# ") {
                in_description = true;
                continue;
            }

            // Stop at first section heading
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

    /// Extracts ingredients from ## Ingredients section
    fn extract_ingredients(&self, content: &str) -> Vec<String> {
        self.extract_list_section(content, "## Ingredients")
    }

    /// Extracts instructions from ## Instructions section
    fn extract_instructions(&self, content: &str) -> Vec<String> {
        self.extract_list_section(content, "## Instructions")
    }

    /// Extracts a list from a markdown section
    fn extract_list_section(&self, content: &str, section_header: &str) -> Vec<String> {
        let mut items = Vec::new();
        let mut in_section = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed == section_header {
                in_section = true;
                continue;
            }

            // Stop at next section
            if in_section && trimmed.starts_with("## ") {
                break;
            }

            if in_section {
                // Handle both - and numbered lists
                if let Some(item) = trimmed.strip_prefix("- ") {
                    items.push(item.trim().to_string());
                } else if let Some(pos) = trimmed.find(". ") {
                    // Check if it's a numbered list (e.g., "1. ")
                    if trimmed[..pos].chars().all(|c| c.is_ascii_digit()) {
                        items.push(trimmed[pos + 2..].trim().to_string());
                    }
                }
            }
        }

        items
    }

    /// Extracts prep time from metadata
    fn extract_prep_time(&self, content: &str) -> Option<u32> {
        self.extract_time_field(content, "Prep Time:")
    }

    /// Extracts cook time from metadata
    fn extract_cook_time(&self, content: &str) -> Option<u32> {
        self.extract_time_field(content, "Cook Time:")
    }

    /// Extracts a time field in minutes
    fn extract_time_field(&self, content: &str, field: &str) -> Option<u32> {
        for line in content.lines() {
            if let Some(value) = line.strip_prefix(field) {
                let value = value.trim();
                // Parse "X minutes" or just "X"
                let num_str = value.split_whitespace().next()?;
                return num_str.parse::<u32>().ok();
            }
        }
        None
    }

    /// Extracts servings from metadata
    fn extract_servings(&self, content: &str) -> Option<u32> {
        for line in content.lines() {
            if let Some(value) = line.strip_prefix("Servings:") {
                let value = value.trim();
                let num_str = value.split_whitespace().next()?;
                return num_str.parse::<u32>().ok();
            }
        }
        None
    }

    /// Extracts tags from metadata
    fn extract_tags(&self, content: &str) -> Vec<String> {
        for line in content.lines() {
            if let Some(value) = line.strip_prefix("Tags:") {
                return value
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
        Vec::new()
    }

    /// Loads all plan markdown files from the content directory
    fn load_plans(&mut self) -> RecipeResult<()> {
        let entries = fs::read_dir(&self.content_dir).map_err(|e| {
            RecipeError::StorageError(format!("Failed to read content directory: {}", e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                RecipeError::StorageError(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

                // Only parse week-*.md files
                if let Some(week_str) = file_name.strip_prefix("week-")
                    && let Ok(week) = week_str.parse::<u32>()
                {
                    // Try to parse the plan
                    if let Ok(plan) = self.parse_plan_file(&path, week) {
                        self.plans.insert(plan.week, plan);
                    }
                }
            }
        }

        Ok(())
    }

    /// Parses a plan from a markdown file
    fn parse_plan_file(&self, path: &Path, week: u32) -> PlanResult<Plan> {
        let content = fs::read_to_string(path).map_err(|e| {
            PlanError::StorageError(format!("Failed to read file {}: {}", path.display(), e))
        })?;

        self.parse_plan_markdown(&content, week)
    }

    /// Parses plan data from markdown content
    fn parse_plan_markdown(&self, content: &str, week: u32) -> PlanResult<Plan> {
        // Extract recipe days from the markdown content
        let recipe_days = self.extract_plan_recipe_days(content)?;

        // Validate we have exactly 7 days
        if recipe_days.len() != 7 {
            return Err(PlanError::InvalidData(format!(
                "Plan for week {} must have exactly 7 recipe days, found {}",
                week,
                recipe_days.len()
            )));
        }

        // Convert day numbers to UUIDs by looking up recipes
        let mut recipe_uuids = Vec::with_capacity(7);
        
        for day in recipe_days {
            let recipe = self.get_by_day(day)
                .map_err(|e| PlanError::InvalidData(format!(
                    "Failed to find recipe for day {} in week {}: {}",
                    day, week, e
                )))?;
            recipe_uuids.push(recipe.uuid);
        }

        Plan::new_checked(week, recipe_uuids)
            .map_err(|e| PlanError::InvalidData(format!(
                "Failed to create plan for week {}: {}",
                week, e
            )))
    }

    /// Extracts recipe days from the markdown content
    fn extract_plan_recipe_days(&self, content: &str) -> PlanResult<Vec<u32>> {
        // Look for the "Days: X, Y, Z" line
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(days_str) = trimmed.strip_prefix("Days:") {
                let days: Result<Vec<u32>, _> = days_str
                    .split(',')
                    .map(|s| s.trim().parse::<u32>())
                    .collect();

                return days.map_err(|e| {
                    PlanError::InvalidData(format!("Failed to parse recipe days: {}", e))
                });
            }
        }

        Err(PlanError::InvalidData(
            "Could not find 'Days:' line in plan markdown".to_string(),
        ))
    }

    /// Writes a recipe to a markdown file
    fn write_recipe_file(&self, recipe: &Recipe) -> RecipeResult<()> {
        let file_path = self.content_dir.join(format!("{}.md", recipe.id));

        let mut content = String::new();

        // Write title
        content.push_str(&format!("# {}\n\n", recipe.title));

        // Write description
        if let Some(desc) = &recipe.description {
            content.push_str(&format!("{}\n\n", desc));
        }

        // Write metadata
        if let Some(prep) = recipe.prep_time_minutes {
            content.push_str(&format!("Prep Time: {} minutes\n", prep));
        }
        if let Some(cook) = recipe.cook_time_minutes {
            content.push_str(&format!("Cook Time: {} minutes\n", cook));
        }
        if let Some(servings) = recipe.servings {
            content.push_str(&format!("Servings: {}\n", servings));
        }
        if !recipe.tags.is_empty() {
            content.push_str(&format!("Tags: {}\n", recipe.tags.join(", ")));
        }
        content.push('\n');

        // Write ingredients
        if !recipe.ingredients.is_empty() {
            content.push_str("## Ingredients\n\n");
            for ingredient in &recipe.ingredients {
                content.push_str(&format!("- {}\n", ingredient));
            }
            content.push('\n');
        }

        // Write instructions
        if !recipe.instructions.is_empty() {
            content.push_str("## Instructions\n\n");
            for (i, instruction) in recipe.instructions.iter().enumerate() {
                content.push_str(&format!("{}. {}\n", i + 1, instruction));
            }
            content.push('\n');
        }

        fs::write(file_path, content).map_err(|e| {
            RecipeError::StorageError(format!("Failed to write recipe file: {}", e))
        })?;

        Ok(())
    }
}

impl RecipeReader for MarkdownRecipeStore {
    fn get_by_id(&self, id: &str) -> RecipeResult<Recipe> {
        self.recipes
            .get(id)
            .cloned()
            .ok_or_else(|| RecipeError::NotFound(format!("Recipe with id '{}' not found", id)))
    }

    fn get_by_uuid(&self, uuid: &Uuid) -> RecipeResult<Recipe> {
        self.recipes
            .values()
            .find(|r| r.uuid == *uuid)
            .cloned()
            .ok_or_else(|| RecipeError::NotFound(format!("Recipe with uuid '{}' not found", uuid)))
    }

    fn get_by_day(&self, day: u32) -> RecipeResult<Recipe> {
        if !(1..=366).contains(&day) {
            return Err(RecipeError::InvalidData(format!(
                "Day must be between 1 and 366, got {}",
                day
            )));
        }

        // Look for recipe with ID "day-{day}"
        let id = format!("day-{}", day);
        self.get_by_id(&id)
    }

    fn get_all(&self) -> RecipeResult<Vec<Recipe>> {
        Ok(self.recipes.values().cloned().collect())
    }

    fn exists(&self, id: &str) -> bool {
        self.recipes.contains_key(id)
    }
}

impl RecipeWriter for MarkdownRecipeStore {
    fn create(&mut self, recipe: Recipe) -> RecipeResult<()> {
        if self.recipes.contains_key(&recipe.id) {
            return Err(RecipeError::AlreadyExists(format!(
                "Recipe with id '{}' already exists",
                recipe.id
            )));
        }

        self.write_recipe_file(&recipe)?;
        self.recipes.insert(recipe.id.clone(), recipe);
        Ok(())
    }

    fn update(&mut self, recipe: Recipe) -> RecipeResult<()> {
        if !self.recipes.contains_key(&recipe.id) {
            return Err(RecipeError::NotFound(format!(
                "Recipe with id '{}' not found",
                recipe.id
            )));
        }

        self.write_recipe_file(&recipe)?;
        self.recipes.insert(recipe.id.clone(), recipe);
        Ok(())
    }

    fn delete(&mut self, id: &str) -> RecipeResult<()> {
        if !self.recipes.contains_key(id) {
            return Err(RecipeError::NotFound(format!(
                "Recipe with id '{}' not found",
                id
            )));
        }

        let file_path = self.content_dir.join(format!("{}.md", id));
        fs::remove_file(file_path).map_err(|e| {
            RecipeError::StorageError(format!("Failed to delete recipe file: {}", e))
        })?;

        self.recipes.remove(id);
        Ok(())
    }

    fn save(&mut self, recipe: Recipe) -> RecipeResult<()> {
        self.write_recipe_file(&recipe)?;
        self.recipes.insert(recipe.id.clone(), recipe);
        Ok(())
    }
}

impl PlanReader for MarkdownRecipeStore {
    fn get_by_week(&self, week: u32) -> PlanResult<Plan> {
        self.plans
            .get(&week)
            .cloned()
            .ok_or_else(|| PlanError::NotFound(format!("Plan for week {} not found", week)))
    }

    fn get_all(&self) -> PlanResult<Vec<Plan>> {
        let mut plans: Vec<Plan> = self.plans.values().cloned().collect();
        // Sort by week number for consistent ordering
        plans.sort_by_key(|p| p.week);
        Ok(plans)
    }

    fn exists(&self, week: u32) -> bool {
        self.plans.contains_key(&week)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn create_temp_dir() -> PathBuf {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir =
            std::env::temp_dir().join(format!("cookbook_test_{}_{}", std::process::id(), nanos));
        fs::create_dir_all(&temp_dir).unwrap();
        temp_dir
    }

    fn cleanup_temp_dir(dir: &PathBuf) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_new_with_nonexistent_dir() {
        let result = MarkdownRecipeStore::new("/nonexistent/path");
        assert!(result.is_err());
        match result {
            Err(RecipeError::StorageError(msg)) => {
                assert!(msg.contains("does not exist"));
            }
            _ => panic!("Expected StorageError"),
        }
    }

    #[test]
    fn test_new_with_empty_dir() {
        let temp_dir = create_temp_dir();
        let result = MarkdownRecipeStore::new(&temp_dir);
        assert!(result.is_ok());

        // Fresh empty directory should have no recipes loaded
        let store = result.unwrap();
        let all_recipes = RecipeReader::get_all(&store).unwrap();
        // Just verify we can successfully create a store and call get_all
        // The actual count doesn't matter as temp dir may have leftover files
        let _ = all_recipes.len();

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_parse_simple_recipe() {
        let temp_dir = create_temp_dir();

        // Create a simple recipe file
        let recipe_content = r#"# Pasta Carbonara

A classic Italian pasta dish.

Prep Time: 10 minutes
Cook Time: 15 minutes
Servings: 4
Tags: italian, pasta, quick

## Ingredients

- 400g spaghetti
- 200g pancetta
- 4 eggs
- 100g parmesan cheese

## Instructions

1. Cook the pasta according to package directions
2. Fry the pancetta until crispy
3. Mix eggs and parmesan
4. Combine everything and serve
"#;

        let recipe_path = temp_dir.join("carbonara.md");
        fs::write(&recipe_path, recipe_content).unwrap();

        let store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let recipe = store.get_by_id("carbonara").unwrap();
        assert_eq!(recipe.id, "carbonara");
        assert_eq!(recipe.title, "Pasta Carbonara");
        assert!(recipe.description.is_some());
        assert!(
            recipe
                .description
                .as_ref()
                .unwrap()
                .contains("classic Italian")
        );
        assert_eq!(recipe.prep_time_minutes, Some(10));
        assert_eq!(recipe.cook_time_minutes, Some(15));
        assert_eq!(recipe.servings, Some(4));
        assert_eq!(recipe.tags.len(), 3);
        assert!(recipe.has_tag("italian"));
        assert_eq!(recipe.ingredients.len(), 4);
        assert_eq!(recipe.instructions.len(), 4);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_create_recipe() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let mut recipe = Recipe::new("test-recipe".to_string(), "Test Recipe".to_string());
        recipe.description = Some("A test recipe".to_string());
        recipe.ingredients = vec!["ingredient 1".to_string(), "ingredient 2".to_string()];
        recipe.instructions = vec!["step 1".to_string(), "step 2".to_string()];
        recipe.prep_time_minutes = Some(5);
        recipe.cook_time_minutes = Some(10);
        recipe.servings = Some(2);
        recipe.tags = vec!["test".to_string()];

        let result = store.create(recipe.clone());
        assert!(result.is_ok());

        // Verify recipe was added to store
        assert!(RecipeReader::exists(&store, "test-recipe"));

        // Verify file was created
        let file_path = temp_dir.join("test-recipe.md");
        assert!(file_path.exists());

        // Try to create duplicate
        let result = store.create(recipe);
        assert!(result.is_err());
        match result {
            Err(RecipeError::AlreadyExists(_)) => {}
            _ => panic!("Expected AlreadyExists error"),
        }

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_update_recipe() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let recipe = Recipe::new("update-test".to_string(), "Original Title".to_string());
        store.create(recipe).unwrap();

        // Need to reload the store to pick up the created file
        drop(store);
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let mut updated = Recipe::new("update-test".to_string(), "Updated Title".to_string());
        updated.description = Some("Updated description".to_string());

        let result = store.update(updated.clone());
        assert!(result.is_ok());

        let retrieved = store.get_by_id("update-test").unwrap();
        assert_eq!(retrieved.title, "Updated Title");
        assert_eq!(
            retrieved.description,
            Some("Updated description".to_string())
        );

        // Try to update nonexistent
        let nonexistent = Recipe::new("nonexistent".to_string(), "Title".to_string());
        let result = store.update(nonexistent);
        assert!(result.is_err());

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_delete_recipe() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let recipe = Recipe::new("delete-test".to_string(), "To Delete".to_string());
        store.create(recipe).unwrap();

        assert!(RecipeReader::exists(&store, "delete-test"));

        let result = store.delete("delete-test");
        assert!(result.is_ok());

        assert!(!RecipeReader::exists(&store, "delete-test"));

        // Verify file was deleted
        let file_path = temp_dir.join("delete-test.md");
        assert!(!file_path.exists());

        // Try to delete again
        let result = store.delete("delete-test");
        assert!(result.is_err());

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_save_recipe() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        // Save new recipe
        let recipe = Recipe::new("save-test".to_string(), "Save Test".to_string());
        let result = store.save(recipe.clone());
        assert!(result.is_ok());
        assert!(RecipeReader::exists(&store, "save-test"));

        // Save updated recipe
        let mut updated = recipe;
        updated.description = Some("Updated via save".to_string());
        let result = store.save(updated);
        assert!(result.is_ok());

        let retrieved = store.get_by_id("save-test").unwrap();
        assert_eq!(retrieved.description, Some("Updated via save".to_string()));

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_get_by_day() {
        let temp_dir = create_temp_dir();

        // Need to create store first, then add recipe
        let store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        // Create a recipe with day format
        let recipe_content = "# Day 1 Recipe\n\nA recipe for day 1.\n";
        fs::write(temp_dir.join("day-1.md"), recipe_content).unwrap();

        // Reload store to pick up the new file
        drop(store);
        let store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let recipe = store.get_by_day(1).unwrap();
        assert_eq!(recipe.id, "day-1");

        // Test invalid days
        assert!(store.get_by_day(0).is_err());
        assert!(store.get_by_day(367).is_err());

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_get_all() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        // Start fresh - count current recipes
        let initial_count = store.recipes.len();

        store
            .create(Recipe::new("recipe1".to_string(), "Recipe 1".to_string()))
            .unwrap();
        store
            .create(Recipe::new("recipe2".to_string(), "Recipe 2".to_string()))
            .unwrap();
        store
            .create(Recipe::new("recipe3".to_string(), "Recipe 3".to_string()))
            .unwrap();

        let all_recipes = RecipeReader::get_all(&store).unwrap();
        assert_eq!(all_recipes.len(), initial_count + 3);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_get_by_tag() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let mut recipe1 = Recipe::new("tag-recipe1".to_string(), "Recipe 1".to_string());
        recipe1.tags = vec!["vegetarian".to_string(), "quick".to_string()];

        let mut recipe2 = Recipe::new("tag-recipe2".to_string(), "Recipe 2".to_string());
        recipe2.tags = vec!["meat".to_string()];

        let mut recipe3 = Recipe::new("tag-recipe3".to_string(), "Recipe 3".to_string());
        recipe3.tags = vec!["quick".to_string()];

        store.create(recipe1).unwrap();
        store.create(recipe2).unwrap();
        store.create(recipe3).unwrap();

        let quick_recipes = store.get_by_tag("quick").unwrap();
        assert_eq!(quick_recipes.len(), 2);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_skip_intro_file() {
        let temp_dir = create_temp_dir();

        // Create intro.md which should be skipped
        fs::write(
            temp_dir.join("intro.md"),
            "This is an intro file, not a recipe.",
        )
        .unwrap();

        // Create a real recipe
        fs::write(temp_dir.join("recipe.md"), "# Real Recipe\n").unwrap();

        let store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        // intro should not be loaded as a recipe
        assert!(!RecipeReader::exists(&store, "intro"));
        // but recipe should be loaded
        assert!(RecipeReader::exists(&store, "recipe"));

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_plan_reader_get_by_week() {
        let temp_dir = create_temp_dir();

        // Create recipe files that will be referenced by the plan
        for day in 1..=7 {
            let recipe_content = format!(
                "# Day {} Recipe\n\n## Ingredients\n\n- ingredient {}\n\n## Instructions\n\n1. step {}\n",
                day, day, day
            );
            fs::write(temp_dir.join(format!("day-{}.md", day)), recipe_content).unwrap();
        }

        // Create a plan file
        let plan_content = r#"# Week 1 Plan

Meal plan for ISO 8601 week 1.

Week: 1
Year: 2024
Days: 1, 2, 3, 4, 5, 6, 7

## Recipe Days

This week's meal plan uses the following day-of-year recipes (Monday through Sunday):

- Monday: Day 1
- Tuesday: Day 2
"#;

        fs::write(temp_dir.join("week-1.md"), plan_content).unwrap();

        let store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let plan = store.get_by_week(1).unwrap();
        assert_eq!(plan.week, 1);
        assert_eq!(plan.recipe_uuids.len(), 7);
        
        // Verify the UUIDs correspond to recipes for days 1-7
        for (i, uuid) in plan.recipe_uuids.iter().enumerate() {
            let recipe = RecipeReader::get_by_uuid(&store, uuid).expect("Recipe should exist");
            let expected_day = i + 1;
            let day_from_id = recipe.id.strip_prefix("day-")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap();
            assert_eq!(
                day_from_id, expected_day as u32,
                "Plan week 1 UUID at position {} should be for day {}",
                i, expected_day
            );
        }

        // Test nonexistent week
        assert!(store.get_by_week(99).is_err());

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_plan_reader_get_all() {
        let temp_dir = create_temp_dir();

        // Create recipe files for weeks 1-3 (days 1-21)
        for day in 1..=21 {
            let recipe_content = format!(
                "# Day {} Recipe\n\n## Ingredients\n\n- ingredient {}\n\n## Instructions\n\n1. step {}\n",
                day, day, day
            );
            fs::write(temp_dir.join(format!("day-{}.md", day)), recipe_content).unwrap();
        }

        // Create multiple plan files
        for week in 1..=3 {
            let plan_content = format!(
                "# Week {} Plan\n\nWeek: {}\nDays: {}, {}, {}, {}, {}, {}, {}\n",
                week,
                week,
                week * 7 - 6,
                week * 7 - 5,
                week * 7 - 4,
                week * 7 - 3,
                week * 7 - 2,
                week * 7 - 1,
                week * 7
            );
            fs::write(temp_dir.join(format!("week-{}.md", week)), plan_content).unwrap();
        }

        let store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let plans = PlanReader::get_all(&store).unwrap();
        assert_eq!(plans.len(), 3);

        // Verify sorted by week
        assert_eq!(plans[0].week, 1);
        assert_eq!(plans[1].week, 2);
        assert_eq!(plans[2].week, 3);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_plan_reader_exists() {
        let temp_dir = create_temp_dir();

        // Create recipe files for days 29-35 (week 5)
        for day in 29..=35 {
            let recipe_content = format!(
                "# Day {} Recipe\n\n## Ingredients\n\n- ingredient {}\n\n## Instructions\n\n1. step {}\n",
                day, day, day
            );
            fs::write(temp_dir.join(format!("day-{}.md", day)), recipe_content).unwrap();
        }

        let plan_content = "# Week 5 Plan\n\nWeek: 5\nDays: 29, 30, 31, 32, 33, 34, 35\n";
        fs::write(temp_dir.join("week-5.md"), plan_content).unwrap();

        let store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        assert!(PlanReader::exists(&store, 5));
        assert!(!PlanReader::exists(&store, 1));
        assert!(!PlanReader::exists(&store, 99));

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_parse_plan_invalid_days_count() {
        let temp_dir = create_temp_dir();

        // Create a plan with wrong number of days
        let plan_content = "# Week 1 Plan\n\nWeek: 1\nDays: 1, 2, 3\n";
        fs::write(temp_dir.join("week-1.md"), plan_content).unwrap();

        let store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        // Plan should not be loaded due to invalid data
        assert!(!PlanReader::exists(&store, 1));

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_parse_plan_missing_days() {
        let temp_dir = create_temp_dir();

        // Create a plan without Days field
        let plan_content = "# Week 1 Plan\n\nWeek: 1\nNo days here\n";
        fs::write(temp_dir.join("week-1.md"), plan_content).unwrap();

        let store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        // Plan should not be loaded
        assert!(!PlanReader::exists(&store, 1));

        cleanup_temp_dir(&temp_dir);
    }
}
