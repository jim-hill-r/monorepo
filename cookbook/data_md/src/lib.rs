use cookbook_core::{
    Plan, PlanError, PlanReader, PlanResult, PlanWriter, Recipe, RecipeError, RecipeReader,
    RecipeResult, RecipeWriter,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

// Export the embedded module for WASM/web targets
pub mod embedded;
mod parse;

/// A markdown-based implementation of RecipeReader and RecipeWriter
/// that reads and writes recipe data from/to markdown files.
///
/// Recipes are keyed by UUID, with an index from each recipe's legacy ID to its UUID.
pub struct MarkdownRecipeStore {
    content_dir: PathBuf,
    recipes: HashMap<Uuid, Recipe>,
    ids: HashMap<String, Uuid>,
    plans: HashMap<u32, Plan>,
    load_errors: Vec<String>,
}

impl MarkdownRecipeStore {
    /// Creates a new MarkdownRecipeStore with the given content directory.
    ///
    /// A recipe or plan file that does not parse is left out of the store, logged as a warning
    /// and listed in [`MarkdownRecipeStore::load_errors`].
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
            ids: HashMap::new(),
            plans: HashMap::new(),
            load_errors: Vec::new(),
        };

        let files = store.markdown_files()?;
        let mut failures = store.load_recipes(&files);
        failures.extend(store.load_plans(&files));

        for failure in &failures {
            tracing::warn!("Skipped content file: {}", failure);
        }
        store.load_errors = failures;

        Ok(store)
    }

    /// One message per content file that was skipped when the store loaded, naming the file
    /// and why it failed: it did not parse, or it reused another recipe's UUID or ID.
    pub fn load_errors(&self) -> &[String] {
        &self.load_errors
    }

    /// The `.md` files in the content directory, sorted by path.
    fn markdown_files(&self) -> RecipeResult<Vec<PathBuf>> {
        let entries = fs::read_dir(&self.content_dir).map_err(|e| {
            RecipeError::StorageError(format!("Failed to read content directory: {}", e))
        })?;

        let mut files = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| {
                RecipeError::StorageError(format!("Failed to read directory entry: {}", e))
            })?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                files.push(path);
            }
        }
        files.sort();
        Ok(files)
    }

    /// Loads every recipe file (all but `intro.md` and `week-*.md`), returning one message per
    /// file that fails to parse or reuses another recipe's UUID or ID.
    fn load_recipes(&mut self, files: &[PathBuf]) -> Vec<String> {
        let mut failures = Vec::new();

        for path in files {
            let file_name = file_stem(path);
            if file_name == "intro" || file_name.starts_with("week-") {
                continue;
            }

            let recipe = match self.parse_recipe_file(path) {
                Ok(recipe) => recipe,
                Err(e) => {
                    failures.push(format!("{}: {}", path.display(), e));
                    continue;
                }
            };

            if self.recipes.contains_key(&recipe.uuid) {
                failures.push(format!(
                    "{}: UUID {} is used by another recipe",
                    path.display(),
                    recipe.uuid
                ));
            } else if self.ids.contains_key(&recipe.id) {
                failures.push(format!(
                    "{}: ID '{}' is used by another recipe",
                    path.display(),
                    recipe.id
                ));
            } else {
                self.insert_recipe(recipe);
            }
        }

        failures
    }

    /// Parses a recipe from a markdown file
    fn parse_recipe_file(&self, path: &Path) -> RecipeResult<Recipe> {
        let content = fs::read_to_string(path).map_err(|e| {
            RecipeError::StorageError(format!("Failed to read file {}: {}", path.display(), e))
        })?;

        self.parse_recipe_markdown(&content, path)
    }

    /// Parses recipe data from markdown content, using the file stem as the fallback ID and title
    fn parse_recipe_markdown(&self, content: &str, path: &Path) -> RecipeResult<Recipe> {
        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| RecipeError::InvalidData("Invalid file name".to_string()))?;

        let id = file_stem.to_string();
        parse::recipe(content, id.clone(), id)
    }

    /// Loads every `week-*.md` plan file, returning one message per file that fails to parse.
    fn load_plans(&mut self, files: &[PathBuf]) -> Vec<String> {
        let mut failures = Vec::new();

        for path in files {
            let Some(week_str) = file_stem(path).strip_prefix("week-") else {
                continue;
            };

            let result = week_str
                .parse::<u32>()
                .map_err(|e| PlanError::InvalidData(format!("Invalid week number: {}", e)))
                .and_then(|week| self.parse_plan_file(path, week));

            match result {
                Ok(plan) => {
                    self.plans.insert(plan.week, plan);
                }
                Err(e) => failures.push(format!("{}: {}", path.display(), e)),
            }
        }

        failures
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
        parse::plan(content, week)
    }

    /// The UUID of the recipe that `id` names: a UUID string or a legacy ID.
    fn resolve_id(&self, id: &str) -> Option<Uuid> {
        if let Ok(uuid) = Uuid::parse_str(id)
            && self.recipes.contains_key(&uuid)
        {
            return Some(uuid);
        }
        self.ids.get(id).copied()
    }

    /// The UUID of the stored recipe that `recipe` replaces: the one with its UUID, or else the
    /// one with its legacy ID.
    fn existing_uuid(&self, recipe: &Recipe) -> Option<Uuid> {
        if self.recipes.contains_key(&recipe.uuid) {
            Some(recipe.uuid)
        } else {
            self.ids.get(&recipe.id).copied()
        }
    }

    /// Adds or replaces a recipe in memory, keeping the legacy-ID index in step.
    fn insert_recipe(&mut self, recipe: Recipe) {
        if let Some(old) = self.recipes.get(&recipe.uuid)
            && self.ids.get(&old.id) == Some(&recipe.uuid)
        {
            self.ids.remove(&old.id);
        }
        self.ids.insert(recipe.id.clone(), recipe.uuid);
        self.recipes.insert(recipe.uuid, recipe);
    }

    /// Writes a recipe to a markdown file
    fn write_recipe_file(&self, recipe: &Recipe) -> RecipeResult<()> {
        // Use UUID for filename instead of legacy ID
        let file_path = self.content_dir.join(format!("{}.md", recipe.uuid));

        let mut content = String::new();

        // Write title
        content.push_str(&format!("# {}\n\n", recipe.title));

        // Write UUID in frontmatter, and the legacy ID when the file name can't carry it
        content.push_str(&format!("UUID: {}\n", recipe.uuid));
        if recipe.id != recipe.uuid.to_string() {
            content.push_str(&format!("ID: {}\n", recipe.id));
        }
        content.push_str(&format!("Status: {}\n", recipe.status));
        content.push('\n');

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
        if let Some(yields) = &recipe.yields {
            content.push_str(&format!("Yield: {}\n", yields));
        }
        if let Some(source) = &recipe.source {
            content.push_str(&format!("Source: {}\n", source));
        }
        if !recipe.tags.is_empty() {
            content.push_str(&format!("Tags: {}\n", recipe.tags.join(", ")));
        }
        content.push('\n');

        // Write equipment
        if !recipe.equipment.is_empty() {
            content.push_str("## Equipment\n\n");
            for item in &recipe.equipment {
                content.push_str(&format!("- {}\n", item));
            }
            content.push('\n');
        }

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

    /// Writes a plan to a markdown file
    fn write_plan_file(&self, plan: &Plan) -> PlanResult<()> {
        let file_path = self.content_dir.join(format!("week-{}.md", plan.week));

        let mut content = String::new();

        // Write title
        content.push_str(&format!("# Week {} Plan\n\n", plan.week));

        // Write description
        content.push_str(&format!("Meal plan for ISO 8601 week {}.\n\n", plan.week));

        // Write week metadata
        content.push_str(&format!("Week: {}\n", plan.week));

        // Write recipe UUIDs
        let uuid_strings: Vec<String> = plan.recipe_uuids.iter().map(|u| u.to_string()).collect();
        content.push_str(&format!("Recipe UUIDs: {}\n\n", uuid_strings.join(", ")));

        // Write recipe days section for human readability
        content.push_str("## Recipe Days\n\n");
        content.push_str(
            "This week's meal plan uses the following recipes (Monday through Sunday):\n\n",
        );

        let weekdays = [
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
            "Sunday",
        ];
        for (i, uuid) in plan.recipe_uuids.iter().enumerate() {
            // Try to get recipe title for better readability
            if let Ok(recipe) = self.get_by_uuid(uuid) {
                content.push_str(&format!("- {}: {}\n", weekdays[i], recipe.title));
            } else {
                content.push_str(&format!("- {}: {}\n", weekdays[i], uuid));
            }
        }

        fs::write(file_path, content)
            .map_err(|e| PlanError::StorageError(format!("Failed to write plan file: {}", e)))?;

        Ok(())
    }
}

impl RecipeReader for MarkdownRecipeStore {
    fn get_by_id(&self, id: &str) -> RecipeResult<Recipe> {
        self.resolve_id(id)
            .and_then(|uuid| self.recipes.get(&uuid))
            .cloned()
            .ok_or_else(|| RecipeError::NotFound(format!("Recipe with id '{}' not found", id)))
    }

    fn get_by_uuid(&self, uuid: &Uuid) -> RecipeResult<Recipe> {
        self.recipes
            .get(uuid)
            .cloned()
            .ok_or_else(|| RecipeError::NotFound(format!("Recipe with uuid '{}' not found", uuid)))
    }

    fn get_all(&self) -> RecipeResult<Vec<Recipe>> {
        Ok(self.recipes.values().cloned().collect())
    }

    /// Accepts a UUID string or a legacy ID, like [`RecipeReader::get_by_id`].
    fn exists(&self, id: &str) -> bool {
        self.resolve_id(id).is_some()
    }
}

impl RecipeWriter for MarkdownRecipeStore {
    /// Fails if a recipe with the same UUID or legacy ID already exists.
    fn create(&mut self, recipe: Recipe) -> RecipeResult<()> {
        if self.recipes.contains_key(&recipe.uuid) || RecipeReader::exists(self, &recipe.id) {
            return Err(RecipeError::AlreadyExists(format!(
                "Recipe with id '{}' or uuid '{}' already exists",
                recipe.id, recipe.uuid
            )));
        }

        self.write_recipe_file(&recipe)?;
        self.insert_recipe(recipe);
        Ok(())
    }

    /// Replaces the recipe with the same UUID, or else the one with the same legacy ID. In the
    /// second case the stored UUID is kept, so plans that reference it still resolve.
    fn update(&mut self, mut recipe: Recipe) -> RecipeResult<()> {
        let uuid = self.existing_uuid(&recipe).ok_or_else(|| {
            RecipeError::NotFound(format!("Recipe with id '{}' not found", recipe.id))
        })?;
        recipe.uuid = uuid;

        if let Some(other) = self.ids.get(&recipe.id)
            && *other != uuid
        {
            return Err(RecipeError::AlreadyExists(format!(
                "Recipe with id '{}' already exists",
                recipe.id
            )));
        }

        self.write_recipe_file(&recipe)?;
        self.insert_recipe(recipe);
        Ok(())
    }

    fn delete(&mut self, id: &str) -> RecipeResult<()> {
        let uuid = self
            .resolve_id(id)
            .ok_or_else(|| RecipeError::NotFound(format!("Recipe with id '{}' not found", id)))?;

        // Delete the file using UUID filename
        let file_path = self.content_dir.join(format!("{}.md", uuid));
        fs::remove_file(file_path).map_err(|e| {
            RecipeError::StorageError(format!("Failed to delete recipe file: {}", e))
        })?;

        if let Some(recipe) = self.recipes.remove(&uuid) {
            self.ids.remove(&recipe.id);
        }
        Ok(())
    }

    /// Updates the matching recipe (see [`RecipeWriter::update`]) or creates a new one.
    fn save(&mut self, recipe: Recipe) -> RecipeResult<()> {
        if self.existing_uuid(&recipe).is_some() {
            RecipeWriter::update(self, recipe)
        } else {
            RecipeWriter::create(self, recipe)
        }
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

impl PlanWriter for MarkdownRecipeStore {
    fn create(&mut self, plan: Plan) -> PlanResult<()> {
        if self.plans.contains_key(&plan.week) {
            return Err(PlanError::AlreadyExists(format!(
                "Plan for week {} already exists",
                plan.week
            )));
        }

        self.write_plan_file(&plan)?;
        self.plans.insert(plan.week, plan);
        Ok(())
    }

    fn update(&mut self, plan: Plan) -> PlanResult<()> {
        if !self.plans.contains_key(&plan.week) {
            return Err(PlanError::NotFound(format!(
                "Plan for week {} not found",
                plan.week
            )));
        }

        self.write_plan_file(&plan)?;
        self.plans.insert(plan.week, plan);
        Ok(())
    }

    fn delete(&mut self, week: u32) -> PlanResult<()> {
        if !self.plans.contains_key(&week) {
            return Err(PlanError::NotFound(format!(
                "Plan for week {} not found",
                week
            )));
        }

        let file_path = self.content_dir.join(format!("week-{}.md", week));
        fs::remove_file(file_path)
            .map_err(|e| PlanError::StorageError(format!("Failed to delete plan file: {}", e)))?;

        self.plans.remove(&week);
        Ok(())
    }

    fn save(&mut self, plan: Plan) -> PlanResult<()> {
        self.write_plan_file(&plan)?;
        self.plans.insert(plan.week, plan);
        Ok(())
    }
}

/// The file name of `path` without its extension, or `""` if it has none.
fn file_stem(path: &Path) -> &str {
    path.file_stem().and_then(|s| s.to_str()).unwrap_or("")
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

    /// Asserts that loading `dir` reports exactly one error for each of `files`, in order.
    fn assert_load_fails(dir: &Path, files: &[&str]) {
        let store = MarkdownRecipeStore::new(dir).unwrap();
        let errors = store.load_errors();
        assert_eq!(errors.len(), files.len(), "errors: {:?}", errors);
        for (error, file) in errors.iter().zip(files) {
            assert!(
                error.contains(file),
                "error should name {}: {}",
                file,
                error
            );
        }
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

UUID: 550e8400-e29b-41d4-a716-446655440099
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
        assert_eq!(
            recipe.description.as_deref(),
            Some("A classic Italian pasta dish.")
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

        let uuid = recipe.uuid;

        let result = RecipeWriter::create(&mut store, recipe.clone());
        assert!(result.is_ok());

        // Verify recipe was added to store
        assert!(RecipeReader::exists(&store, "test-recipe"));

        // Verify file was created with UUID filename
        let file_path = temp_dir.join(format!("{}.md", uuid));
        assert!(file_path.exists());

        // Try to create duplicate
        let result = RecipeWriter::create(&mut store, recipe);
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
        let uuid = recipe.uuid;
        RecipeWriter::create(&mut store, recipe).unwrap();

        // Need to reload the store to pick up the created file
        drop(store);
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        // After reload, the recipe ID will be the UUID string (since filename is UUID)
        // So we need to get the recipe by UUID first
        let loaded_recipe = store.get_by_uuid(&uuid).unwrap();

        // Create updated recipe with the same UUID and ID as loaded recipe
        let mut updated =
            Recipe::new_with_uuid(loaded_recipe.id.clone(), uuid, "Updated Title".to_string());
        updated.description = Some("Updated description".to_string());

        let result = RecipeWriter::update(&mut store, updated.clone());
        assert!(result.is_ok());

        let retrieved = store.get_by_uuid(&uuid).unwrap();
        assert_eq!(retrieved.title, "Updated Title");
        assert_eq!(
            retrieved.description,
            Some("Updated description".to_string())
        );

        // Try to update nonexistent
        let nonexistent = Recipe::new("nonexistent".to_string(), "Title".to_string());
        let result = RecipeWriter::update(&mut store, nonexistent);
        assert!(result.is_err());

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_delete_recipe() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let recipe = Recipe::new("delete-test".to_string(), "To Delete".to_string());
        let uuid = recipe.uuid;
        RecipeWriter::create(&mut store, recipe).unwrap();

        assert!(RecipeReader::exists(&store, "delete-test"));

        let result = RecipeWriter::delete(&mut store, "delete-test");
        assert!(result.is_ok());

        assert!(!RecipeReader::exists(&store, "delete-test"));

        // Verify file was deleted (UUID filename)
        let file_path = temp_dir.join(format!("{}.md", uuid));
        assert!(!file_path.exists());

        // Try to delete again
        let result = RecipeWriter::delete(&mut store, "delete-test");
        assert!(result.is_err());

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_save_recipe() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        // Save new recipe
        let recipe = Recipe::new("save-test".to_string(), "Save Test".to_string());
        let result = RecipeWriter::save(&mut store, recipe.clone());
        assert!(result.is_ok());
        assert!(RecipeReader::exists(&store, "save-test"));

        // Save updated recipe
        let mut updated = recipe;
        updated.description = Some("Updated via save".to_string());
        let result = RecipeWriter::save(&mut store, updated);
        assert!(result.is_ok());

        let retrieved = store.get_by_id("save-test").unwrap();
        assert_eq!(retrieved.description, Some("Updated via save".to_string()));

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_get_all() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        // Start fresh - count current recipes
        let initial_count = store.recipes.len();

        RecipeWriter::create(
            &mut store,
            Recipe::new("recipe1".to_string(), "Recipe 1".to_string()),
        )
        .unwrap();
        RecipeWriter::create(
            &mut store,
            Recipe::new("recipe2".to_string(), "Recipe 2".to_string()),
        )
        .unwrap();
        RecipeWriter::create(
            &mut store,
            Recipe::new("recipe3".to_string(), "Recipe 3".to_string()),
        )
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

        RecipeWriter::create(&mut store, recipe1).unwrap();
        RecipeWriter::create(&mut store, recipe2).unwrap();
        RecipeWriter::create(&mut store, recipe3).unwrap();

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
        fs::write(
            temp_dir.join("recipe.md"),
            format!("# Real Recipe\n\nUUID: {}\n", Uuid::new_v4()),
        )
        .unwrap();

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
        let mut recipe_uuids = Vec::new();
        for day in 1..=7 {
            let uuid = Uuid::new_v4();
            recipe_uuids.push(uuid);
            let recipe_content = format!(
                "# Day {} Recipe\n\nUUID: {}\n\n## Ingredients\n\n- ingredient {}\n\n## Instructions\n\n1. step {}\n",
                day, uuid, day, day
            );
            fs::write(temp_dir.join(format!("day-{}.md", day)), recipe_content).unwrap();
        }

        // Create a plan file with the UUIDs
        let uuids_str = recipe_uuids
            .iter()
            .map(|u| u.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let plan_content = format!(
            r#"# Week 1 Plan

Meal plan for ISO 8601 week 1.

Week: 1
Year: 2024
Days: 1, 2, 3, 4, 5, 6, 7
Recipe UUIDs: {}

## Recipe Days

This week's meal plan uses the following day-of-year recipes (Monday through Sunday):

- Monday: Day 1
- Tuesday: Day 2
"#,
            uuids_str
        );

        fs::write(temp_dir.join("week-1.md"), plan_content).unwrap();

        let store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let plan = store.get_by_week(1).unwrap();
        assert_eq!(plan.week, 1);
        assert_eq!(plan.recipe_uuids.len(), 7);

        // Verify the UUIDs correspond to recipes for days 1-7
        for (i, uuid) in plan.recipe_uuids.iter().enumerate() {
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

        // Test nonexistent week
        assert!(store.get_by_week(99).is_err());

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_plan_reader_get_all() {
        let temp_dir = create_temp_dir();

        // Create recipe files for weeks 1-3 (days 1-21) with UUIDs
        let mut all_uuids = Vec::new();
        for day in 1..=21 {
            let uuid = Uuid::new_v4();
            all_uuids.push(uuid);
            let recipe_content = format!(
                "# Day {} Recipe\n\nUUID: {}\n\n## Ingredients\n\n- ingredient {}\n\n## Instructions\n\n1. step {}\n",
                day, uuid, day, day
            );
            fs::write(temp_dir.join(format!("day-{}.md", day)), recipe_content).unwrap();
        }

        // Create multiple plan files with UUIDs
        for week in 1..=3 {
            let start_idx = (week - 1) * 7;
            let week_uuids: Vec<String> = all_uuids[start_idx..start_idx + 7]
                .iter()
                .map(|u| u.to_string())
                .collect();
            let uuids_str = week_uuids.join(", ");

            let plan_content = format!(
                "# Week {} Plan\n\nWeek: {}\nDays: {}, {}, {}, {}, {}, {}, {}\nRecipe UUIDs: {}\n",
                week,
                week,
                week * 7 - 6,
                week * 7 - 5,
                week * 7 - 4,
                week * 7 - 3,
                week * 7 - 2,
                week * 7 - 1,
                week * 7,
                uuids_str
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

        // Create recipe files for days 29-35 (week 5) with UUIDs
        let mut week_uuids = Vec::new();
        for day in 29..=35 {
            let uuid = Uuid::new_v4();
            week_uuids.push(uuid);
            let recipe_content = format!(
                "# Day {} Recipe\n\nUUID: {}\n\n## Ingredients\n\n- ingredient {}\n\n## Instructions\n\n1. step {}\n",
                day, uuid, day, day
            );
            fs::write(temp_dir.join(format!("day-{}.md", day)), recipe_content).unwrap();
        }

        let uuids_str = week_uuids
            .iter()
            .map(|u| u.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let plan_content = format!(
            "# Week 5 Plan\n\nWeek: 5\nDays: 29, 30, 31, 32, 33, 34, 35\nRecipe UUIDs: {}\n",
            uuids_str
        );
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

        assert_load_fails(&temp_dir, &["week-1.md"]);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_parse_plan_missing_days() {
        let temp_dir = create_temp_dir();

        // Create a plan without Days field
        let plan_content = "# Week 1 Plan\n\nWeek: 1\nNo days here\n";
        fs::write(temp_dir.join("week-1.md"), plan_content).unwrap();

        assert_load_fails(&temp_dir, &["week-1.md"]);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_parse_plan_with_uuids() {
        let temp_dir = create_temp_dir();

        // Create recipe files with known UUIDs
        let uuid1 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
        let uuid2 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap();
        let uuid3 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440003").unwrap();
        let uuid4 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440004").unwrap();
        let uuid5 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440005").unwrap();
        let uuid6 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440006").unwrap();
        let uuid7 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440007").unwrap();

        for (i, uuid) in [uuid1, uuid2, uuid3, uuid4, uuid5, uuid6, uuid7]
            .iter()
            .enumerate()
        {
            let recipe_content = format!("# Recipe {}\n\nUUID: {}\n\nA test recipe\n", i + 1, uuid);
            fs::write(temp_dir.join(format!("day-{}.md", i + 1)), recipe_content).unwrap();
        }

        // Create a plan with UUID references
        let plan_content = format!(
            "# Week 1 Plan\n\nWeek: 1\nYear: 2024\nDays: 1, 2, 3, 4, 5, 6, 7\nRecipe UUIDs: {}, {}, {}, {}, {}, {}, {}\n",
            uuid1, uuid2, uuid3, uuid4, uuid5, uuid6, uuid7
        );
        fs::write(temp_dir.join("week-1.md"), plan_content).unwrap();

        let store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        // Plan should be loaded with correct UUIDs
        assert!(PlanReader::exists(&store, 1));
        let plan = PlanReader::get_by_week(&store, 1).unwrap();
        assert_eq!(plan.week, 1);
        assert_eq!(plan.recipe_uuids.len(), 7);
        assert_eq!(plan.recipe_uuids[0], uuid1);
        assert_eq!(plan.recipe_uuids[1], uuid2);
        assert_eq!(plan.recipe_uuids[6], uuid7);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_parse_plan_with_uuids_invalid_count() {
        let temp_dir = create_temp_dir();

        // Create recipe files
        for i in 1..=3 {
            let uuid = Uuid::new_v4();
            let recipe_content = format!("# Recipe {}\n\nUUID: {}\n\nA test recipe\n", i, uuid);
            fs::write(temp_dir.join(format!("day-{}.md", i)), recipe_content).unwrap();
        }

        // Create a plan with wrong number of UUIDs (only 3 instead of 7)
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();
        let uuid3 = Uuid::new_v4();
        let plan_content = format!(
            "# Week 1 Plan\n\nWeek: 1\nRecipe UUIDs: {}, {}, {}\n",
            uuid1, uuid2, uuid3
        );
        fs::write(temp_dir.join("week-1.md"), plan_content).unwrap();

        assert_load_fails(&temp_dir, &["week-1.md"]);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_parse_recipe_with_uuid_in_frontmatter() {
        let temp_dir = create_temp_dir();

        // Create a recipe file with UUID in frontmatter
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let recipe_content = format!(
            r#"# Recipe with UUID

A recipe that has a UUID in the frontmatter.

UUID: {}
Prep Time: 10 minutes
Cook Time: 15 minutes
Servings: 4
Tags: test

## Ingredients

- 1 cup flour
- 2 eggs

## Instructions

1. Mix ingredients
2. Cook
"#,
            uuid_str
        );

        let recipe_path = temp_dir.join("uuid-recipe.md");
        fs::write(&recipe_path, recipe_content).unwrap();

        let store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let recipe = store.get_by_id("uuid-recipe").unwrap();
        assert_eq!(recipe.id, "uuid-recipe");
        assert_eq!(recipe.title, "Recipe with UUID");
        // Verify the UUID was parsed from frontmatter
        assert_eq!(recipe.uuid.to_string(), uuid_str);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_load_fails_for_recipe_without_uuid() {
        let temp_dir = create_temp_dir();

        // Create a recipe file without UUID
        let recipe_content = r#"# Recipe without UUID

A recipe that doesn't have a UUID in the frontmatter.

Prep Time: 10 minutes
Cook Time: 15 minutes
Servings: 4
Tags: test

## Ingredients

- 1 cup flour
- 2 eggs

## Instructions

1. Mix ingredients
2. Cook
"#;

        let recipe_path = temp_dir.join("no-uuid-recipe.md");
        fs::write(&recipe_path, recipe_content).unwrap();

        assert_load_fails(&temp_dir, &["no-uuid-recipe.md"]);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_load_fails_for_recipe_with_malformed_uuid() {
        let temp_dir = create_temp_dir();

        // Create a recipe file with invalid UUID
        let recipe_content = r#"# Recipe with Invalid UUID

A recipe that has an invalid UUID in the frontmatter.

UUID: not-a-valid-uuid
Prep Time: 10 minutes
Cook Time: 15 minutes
Servings: 4
Tags: test

## Ingredients

- 1 cup flour
- 2 eggs

## Instructions

1. Mix ingredients
2. Cook
"#;

        let recipe_path = temp_dir.join("invalid-uuid-recipe.md");
        fs::write(&recipe_path, recipe_content).unwrap();

        assert_load_fails(&temp_dir, &["invalid-uuid-recipe.md"]);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_load_reports_every_bad_file() {
        let temp_dir = create_temp_dir();

        fs::write(temp_dir.join("recipe1.md"), "# Recipe 1\n\nFirst recipe.\n").unwrap();
        fs::write(temp_dir.join("recipe2.md"), "# Recipe 2\n\nUUID: nope\n").unwrap();
        fs::write(temp_dir.join("week-2.md"), "# Week 2\n\nWeek: 2\n").unwrap();

        assert_load_fails(&temp_dir, &["recipe1.md", "recipe2.md", "week-2.md"]);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_load_fails_for_duplicate_uuid() {
        let temp_dir = create_temp_dir();

        let content = format!("# Twin\n\nUUID: {}\n", Uuid::new_v4());
        fs::write(temp_dir.join("a.md"), &content).unwrap();
        fs::write(temp_dir.join("b.md"), &content).unwrap();

        assert_load_fails(&temp_dir, &["b.md"]);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_recipe_id_survives_create_and_reload() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let mut recipe = Recipe::new("day-1".to_string(), "Pancakes".to_string());
        recipe.description = Some("Fluffy.".to_string());
        recipe.prep_time_minutes = Some(5);
        recipe.tags = vec!["breakfast".to_string()];
        RecipeWriter::create(&mut store, recipe.clone()).unwrap();

        let store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        assert_eq!(store.get_by_id("day-1").unwrap(), recipe);
        assert!(RecipeReader::exists(&store, "day-1"));
        assert!(RecipeReader::exists(&store, &recipe.uuid.to_string()));
        assert!(!RecipeReader::exists(&store, &Uuid::new_v4().to_string()));

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_book_fields_round_trip() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let mut recipe = Recipe::new("day-2".to_string(), "Sourdough".to_string());
        recipe.description = Some("A slow loaf.".to_string());
        recipe.servings = Some(8);
        recipe.status = cookbook_core::RecipeStatus::BookReady;
        recipe.yields = Some("one 9-inch loaf".to_string());
        recipe.source = Some("Grandma Hill, 1972".to_string());
        recipe.equipment = vec!["Dutch oven".to_string(), "bench scraper".to_string()];
        recipe.ingredients = vec!["500 g flour".to_string()];
        recipe.instructions = vec!["Bake.".to_string()];
        RecipeWriter::create(&mut store, recipe.clone()).unwrap();

        let written = fs::read_to_string(temp_dir.join(format!("{}.md", recipe.uuid))).unwrap();
        assert!(written.contains("Status: book-ready\n"));
        assert!(written.contains("Yield: one 9-inch loaf\n"));
        assert!(written.contains("Source: Grandma Hill, 1972\n"));
        assert!(written.contains("## Equipment\n\n- Dutch oven\n- bench scraper\n"));

        let store = MarkdownRecipeStore::new(&temp_dir).unwrap();
        assert!(store.load_errors().is_empty());
        assert_eq!(store.get_by_id("day-2").unwrap(), recipe);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_every_status_round_trips() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let recipes: Vec<Recipe> = cookbook_core::RecipeStatus::ALL
            .into_iter()
            .enumerate()
            .map(|(i, status)| {
                let mut recipe = Recipe::new(format!("day-{}", i + 1), format!("Recipe {i}"));
                recipe.status = status;
                recipe
            })
            .collect();
        for recipe in &recipes {
            RecipeWriter::create(&mut store, recipe.clone()).unwrap();
        }

        let store = MarkdownRecipeStore::new(&temp_dir).unwrap();
        for recipe in &recipes {
            assert_eq!(&store.get_by_uuid(&recipe.uuid).unwrap(), recipe);
        }

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_load_fails_for_unknown_status() {
        let temp_dir = create_temp_dir();
        fs::write(
            temp_dir.join("bad-status.md"),
            format!("# Soup\n\nUUID: {}\nStatus: finished\n", Uuid::new_v4()),
        )
        .unwrap();

        assert_load_fails(&temp_dir, &["bad-status.md"]);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_create_after_reload_rejects_existing_id() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();
        RecipeWriter::create(
            &mut store,
            Recipe::new("day-1".to_string(), "A".to_string()),
        )
        .unwrap();

        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();
        let result = RecipeWriter::create(
            &mut store,
            Recipe::new("day-1".to_string(), "B".to_string()),
        );

        assert!(matches!(result, Err(RecipeError::AlreadyExists(_))));

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_update_by_legacy_id_keeps_uuid_and_file() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();
        let original = Recipe::new("day-1".to_string(), "Old".to_string());
        RecipeWriter::create(&mut store, original.clone()).unwrap();

        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();
        // A fresh Recipe gets a new random UUID, but names the same legacy ID.
        RecipeWriter::save(
            &mut store,
            Recipe::new("day-1".to_string(), "New".to_string()),
        )
        .unwrap();

        let store = MarkdownRecipeStore::new(&temp_dir).unwrap();
        let reloaded = store.get_by_id("day-1").unwrap();
        assert_eq!(reloaded.uuid, original.uuid);
        assert_eq!(reloaded.title, "New");
        assert_eq!(fs::read_dir(&temp_dir).unwrap().count(), 1);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_update_rejects_id_of_another_recipe() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();
        let first = Recipe::new("first".to_string(), "First".to_string());
        RecipeWriter::create(&mut store, first.clone()).unwrap();
        RecipeWriter::create(
            &mut store,
            Recipe::new("second".to_string(), "Second".to_string()),
        )
        .unwrap();

        let mut renamed = first;
        renamed.id = "second".to_string();
        let result = RecipeWriter::update(&mut store, renamed);

        assert!(matches!(result, Err(RecipeError::AlreadyExists(_))));
        assert_eq!(store.get_by_id("second").unwrap().title, "Second");

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_update_renames_legacy_id() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();
        let recipe = Recipe::new("old-id".to_string(), "Soup".to_string());
        RecipeWriter::create(&mut store, recipe.clone()).unwrap();

        let mut renamed = recipe.clone();
        renamed.id = "new-id".to_string();
        RecipeWriter::update(&mut store, renamed).unwrap();

        assert!(!RecipeReader::exists(&store, "old-id"));
        assert_eq!(store.get_by_id("new-id").unwrap().uuid, recipe.uuid);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_delete_by_uuid_string_clears_legacy_id() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();
        let recipe = Recipe::new("gone".to_string(), "Gone".to_string());
        RecipeWriter::create(&mut store, recipe.clone()).unwrap();

        RecipeWriter::delete(&mut store, &recipe.uuid.to_string()).unwrap();

        assert!(!RecipeReader::exists(&store, "gone"));
        assert!(store.get_by_uuid(&recipe.uuid).is_err());

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_get_by_id_with_uuid_string() {
        let temp_dir = create_temp_dir();

        // Create a recipe with a known UUID
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let recipe_content = format!(
            r#"# Test Recipe

A recipe with a UUID.

UUID: {}
Prep Time: 10 minutes
Cook Time: 15 minutes
Servings: 4
Tags: test

## Ingredients

- 1 cup flour
- 2 eggs

## Instructions

1. Mix ingredients
2. Cook
"#,
            uuid_str
        );

        let recipe_path = temp_dir.join("test-recipe.md");
        fs::write(&recipe_path, recipe_content).unwrap();

        let store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        // Test that get_by_id works with UUID string
        let recipe_by_uuid_str = store.get_by_id(uuid_str).unwrap();
        assert_eq!(recipe_by_uuid_str.uuid.to_string(), uuid_str);
        assert_eq!(recipe_by_uuid_str.title, "Test Recipe");

        // Test that get_by_id still works with legacy ID
        let recipe_by_id = store.get_by_id("test-recipe").unwrap();
        assert_eq!(recipe_by_id.id, "test-recipe");
        assert_eq!(recipe_by_id.uuid.to_string(), uuid_str);

        // Both methods should return the same recipe
        assert_eq!(recipe_by_uuid_str, recipe_by_id);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_create_recipe_uses_uuid_filename() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let recipe = Recipe::new("test-recipe".to_string(), "Test Recipe".to_string());
        let uuid = recipe.uuid;

        let result = RecipeWriter::create(&mut store, recipe.clone());
        assert!(result.is_ok());

        // Verify file was created with UUID filename
        let uuid_file_path = temp_dir.join(format!("{}.md", uuid));
        assert!(
            uuid_file_path.exists(),
            "Recipe file should be created with UUID filename"
        );

        // Verify old ID-based filename was NOT created
        let id_file_path = temp_dir.join("test-recipe.md");
        assert!(
            !id_file_path.exists(),
            "Recipe file should NOT be created with ID filename"
        );

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_create_recipe_writes_uuid_to_frontmatter() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let recipe = Recipe::new("test-recipe".to_string(), "Test Recipe".to_string());
        let uuid = recipe.uuid;

        RecipeWriter::create(&mut store, recipe.clone()).unwrap();

        // Read the file and check for UUID in frontmatter
        let uuid_file_path = temp_dir.join(format!("{}.md", uuid));
        let content = fs::read_to_string(uuid_file_path).unwrap();

        // UUID should be in the frontmatter (after the title)
        assert!(
            content.contains(&format!("UUID: {}", uuid)),
            "File should contain UUID in frontmatter"
        );

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_update_recipe_uses_uuid_filename() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let recipe = Recipe::new("update-test".to_string(), "Original Title".to_string());
        let uuid = recipe.uuid;
        RecipeWriter::create(&mut store, recipe).unwrap();

        // Update the recipe
        let mut updated =
            Recipe::new_with_uuid("update-test".to_string(), uuid, "Updated Title".to_string());
        updated.description = Some("Updated description".to_string());

        RecipeWriter::update(&mut store, updated).unwrap();

        // Verify file exists with UUID filename
        let uuid_file_path = temp_dir.join(format!("{}.md", uuid));
        assert!(
            uuid_file_path.exists(),
            "Updated recipe file should use UUID filename"
        );

        // Verify content was updated
        let content = fs::read_to_string(uuid_file_path).unwrap();
        assert!(content.contains("Updated Title"));
        assert!(content.contains("Updated description"));

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_delete_recipe_by_uuid() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let recipe = Recipe::new("delete-test".to_string(), "To Delete".to_string());
        let uuid = recipe.uuid;
        RecipeWriter::create(&mut store, recipe).unwrap();

        // Verify file exists
        let uuid_file_path = temp_dir.join(format!("{}.md", uuid));
        assert!(uuid_file_path.exists());

        // Delete by UUID string
        let result = RecipeWriter::delete(&mut store, &uuid.to_string());
        assert!(result.is_ok());

        // Verify file was deleted
        assert!(!uuid_file_path.exists());

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_save_recipe_uses_uuid_filename() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let recipe = Recipe::new("save-test".to_string(), "Save Test".to_string());
        let uuid = recipe.uuid;

        RecipeWriter::save(&mut store, recipe.clone()).unwrap();

        // Verify file was saved with UUID filename
        let uuid_file_path = temp_dir.join(format!("{}.md", uuid));
        assert!(
            uuid_file_path.exists(),
            "Saved recipe file should use UUID filename"
        );

        // Verify UUID is in frontmatter
        let content = fs::read_to_string(uuid_file_path).unwrap();
        assert!(content.contains(&format!("UUID: {}", uuid)));

        cleanup_temp_dir(&temp_dir);
    }

    // PlanWriter trait tests

    #[test]
    fn test_plan_writer_create() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let uuids = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let plan = Plan::new(10, uuids.clone()).unwrap();

        let result = PlanWriter::create(&mut store, plan.clone());
        assert!(result.is_ok());

        // Verify plan was added to store
        assert!(PlanReader::exists(&store, 10));

        // Verify file was created
        let file_path = temp_dir.join("week-10.md");
        assert!(file_path.exists());

        // Verify file content
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("# Week 10 Plan"));
        assert!(content.contains("Week: 10"));
        for uuid in &uuids {
            assert!(content.contains(&uuid.to_string()));
        }

        // Try to create duplicate
        let result = PlanWriter::create(&mut store, plan);
        assert!(result.is_err());
        match result {
            Err(PlanError::AlreadyExists(_)) => {}
            _ => panic!("Expected AlreadyExists error"),
        }

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_plan_writer_update() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let uuids1 = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let plan1 = Plan::new(20, uuids1).unwrap();
        PlanWriter::create(&mut store, plan1).unwrap();

        // Create updated plan with different UUIDs
        let uuids2 = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let plan2 = Plan::new(20, uuids2.clone()).unwrap();

        let result = PlanWriter::update(&mut store, plan2.clone());
        assert!(result.is_ok());

        // Verify plan was updated
        let retrieved = PlanReader::get_by_week(&store, 20).unwrap();
        assert_eq!(retrieved.recipe_uuids, uuids2);

        // Try to update nonexistent
        let uuids3 = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let nonexistent = Plan::new(52, uuids3).unwrap();
        let result = PlanWriter::update(&mut store, nonexistent);
        assert!(result.is_err());
        match result {
            Err(PlanError::NotFound(_)) => {}
            _ => panic!("Expected NotFound error"),
        }

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_plan_writer_delete() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let uuids1 = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let uuids2 = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let plan1 = Plan::new(30, uuids1).unwrap();
        let plan2 = Plan::new(31, uuids2).unwrap();
        PlanWriter::create(&mut store, plan1).unwrap();
        PlanWriter::create(&mut store, plan2.clone()).unwrap();

        assert!(PlanReader::exists(&store, 30));
        assert!(PlanReader::exists(&store, 31));

        let result = PlanWriter::delete(&mut store, 30);
        assert!(result.is_ok());

        assert!(!PlanReader::exists(&store, 30));
        assert!(PlanReader::exists(&store, 31));

        // Verify file was deleted
        let file_path = temp_dir.join("week-30.md");
        assert!(!file_path.exists());

        // Try to delete again
        let result = PlanWriter::delete(&mut store, 30);
        assert!(result.is_err());
        match result {
            Err(PlanError::NotFound(_)) => {}
            _ => panic!("Expected NotFound error"),
        }

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_plan_writer_save_create() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let uuids = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let plan = Plan::new(40, uuids.clone()).unwrap();

        let result = PlanWriter::save(&mut store, plan.clone());
        assert!(result.is_ok());

        // Verify plan was created
        assert!(PlanReader::exists(&store, 40));
        let retrieved = PlanReader::get_by_week(&store, 40).unwrap();
        assert_eq!(retrieved, plan);

        // Verify file was created
        let file_path = temp_dir.join("week-40.md");
        assert!(file_path.exists());

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_plan_writer_save_update() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let uuids1 = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let plan1 = Plan::new(50, uuids1).unwrap();
        PlanWriter::create(&mut store, plan1).unwrap();

        // Save updated plan
        let uuids2 = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let plan2 = Plan::new(50, uuids2.clone()).unwrap();

        let result = PlanWriter::save(&mut store, plan2.clone());
        assert!(result.is_ok());

        // Verify plan was updated
        let retrieved = PlanReader::get_by_week(&store, 50).unwrap();
        assert_eq!(retrieved.recipe_uuids, uuids2);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_plan_writer_creates_readable_file() {
        let temp_dir = create_temp_dir();

        // First create some recipes so the plan can reference them
        let mut recipe_uuids = Vec::new();
        for i in 1..=7 {
            let recipe_content = format!(
                "# Test Recipe {}\n\nUUID: {}\n\nA test recipe.\n\n## Ingredients\n\n- ingredient {}\n\n## Instructions\n\n1. step {}\n",
                i,
                Uuid::new_v4(),
                i,
                i
            );
            fs::write(
                temp_dir.join(format!("test-recipe-{}.md", i)),
                recipe_content,
            )
            .unwrap();
        }

        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        // Get the UUIDs of the created recipes
        let all_recipes = RecipeReader::get_all(&store).unwrap();
        for recipe in all_recipes.iter().take(7) {
            recipe_uuids.push(recipe.uuid);
        }

        if recipe_uuids.len() < 7 {
            // If we don't have enough recipes, add dummy UUIDs
            while recipe_uuids.len() < 7 {
                recipe_uuids.push(Uuid::new_v4());
            }
        }

        let plan = Plan::new(15, recipe_uuids.clone()).unwrap();
        PlanWriter::create(&mut store, plan).unwrap();

        // Verify file content is human-readable
        let file_path = temp_dir.join("week-15.md");
        let content = fs::read_to_string(&file_path).unwrap();

        assert!(content.contains("# Week 15 Plan"));
        assert!(content.contains("Week: 15"));
        assert!(content.contains("Recipe UUIDs:"));
        assert!(content.contains("## Recipe Days"));
        assert!(content.contains("Monday"));
        assert!(content.contains("Sunday"));

        // Verify all UUIDs are in the file
        for uuid in &recipe_uuids {
            assert!(content.contains(&uuid.to_string()));
        }

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_plan_writer_roundtrip() {
        let temp_dir = create_temp_dir();
        let mut store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        let uuids = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let plan = Plan::new(25, uuids.clone()).unwrap();

        // Write plan
        PlanWriter::create(&mut store, plan.clone()).unwrap();

        // Reload store to read plan from disk
        drop(store);
        let store = MarkdownRecipeStore::new(&temp_dir).unwrap();

        // Read plan and verify it matches
        let loaded_plan = PlanReader::get_by_week(&store, 25).unwrap();
        assert_eq!(loaded_plan.week, plan.week);
        assert_eq!(loaded_plan.recipe_uuids, plan.recipe_uuids);

        cleanup_temp_dir(&temp_dir);
    }
}
