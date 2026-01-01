use std::fmt;
use uuid::Uuid;

/// Errors that can occur when working with recipes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecipeError {
    /// Recipe was not found
    NotFound(String),
    /// I/O or storage error occurred
    StorageError(String),
    /// Recipe data is invalid or malformed
    InvalidData(String),
    /// Recipe already exists (for create operations)
    AlreadyExists(String),
}

impl fmt::Display for RecipeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecipeError::NotFound(msg) => write!(f, "Recipe not found: {}", msg),
            RecipeError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            RecipeError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            RecipeError::AlreadyExists(msg) => write!(f, "Recipe already exists: {}", msg),
        }
    }
}

impl std::error::Error for RecipeError {}

/// Result type for recipe operations
pub type RecipeResult<T> = Result<T, RecipeError>;

/// Trait for reading recipe information from a data source
pub trait RecipeReader {
    /// Get a recipe by its unique identifier
    fn get_by_id(&self, id: &str) -> RecipeResult<Recipe>;

    /// Get a recipe by its UUID
    fn get_by_uuid(&self, uuid: &Uuid) -> RecipeResult<Recipe>;

    /// Get a recipe by day of the year (1-365, or 1-366 in leap years)
    fn get_by_day(&self, day: u32) -> RecipeResult<Recipe>;

    /// Get all recipes
    fn get_all(&self) -> RecipeResult<Vec<Recipe>>;

    /// Check if a recipe exists by ID
    ///
    /// This is a required method to allow implementations to optimize
    /// existence checks without loading the entire recipe.
    fn exists(&self, id: &str) -> bool;

    /// Get recipes by tag
    fn get_by_tag(&self, tag: &str) -> RecipeResult<Vec<Recipe>> {
        let all_recipes = self.get_all()?;
        Ok(all_recipes.into_iter().filter(|r| r.has_tag(tag)).collect())
    }
}

/// Trait for writing recipe information to a data source
pub trait RecipeWriter {
    /// Create a new recipe
    /// Returns an error if a recipe with the same ID already exists
    fn create(&mut self, recipe: Recipe) -> RecipeResult<()>;

    /// Update an existing recipe
    /// Returns an error if the recipe doesn't exist
    fn update(&mut self, recipe: Recipe) -> RecipeResult<()>;

    /// Delete a recipe by ID
    /// Returns an error if the recipe doesn't exist
    fn delete(&mut self, id: &str) -> RecipeResult<()>;

    /// Create or update a recipe (upsert)
    ///
    /// This is a required method to allow implementations to optimize
    /// the upsert operation without unnecessary clones or checks.
    fn save(&mut self, recipe: Recipe) -> RecipeResult<()>;
}

/// Represents a recipe with all its associated information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recipe {
    /// Unique identifier for the recipe (legacy, use uuid instead)
    pub id: String,
    /// UUID for the recipe (primary identifier)
    pub uuid: Uuid,
    /// Title of the recipe
    pub title: String,
    /// Description of the recipe
    pub description: Option<String>,
    /// List of ingredients with quantities
    pub ingredients: Vec<String>,
    /// Step-by-step instructions
    pub instructions: Vec<String>,
    /// Preparation time in minutes
    pub prep_time_minutes: Option<u32>,
    /// Cooking time in minutes
    pub cook_time_minutes: Option<u32>,
    /// Number of servings this recipe makes
    pub servings: Option<u32>,
    /// Tags for categorization (e.g., "vegetarian", "dessert", "quick")
    pub tags: Vec<String>,
}

impl Recipe {
    /// Creates a new recipe with required fields and a generated UUID
    pub fn new(id: String, title: String) -> Self {
        Self {
            id,
            uuid: Uuid::new_v4(),
            title,
            description: None,
            ingredients: Vec::new(),
            instructions: Vec::new(),
            prep_time_minutes: None,
            cook_time_minutes: None,
            servings: None,
            tags: Vec::new(),
        }
    }

    /// Creates a new recipe with a specific UUID (for loading from storage)
    pub fn new_with_uuid(id: String, uuid: Uuid, title: String) -> Self {
        Self {
            id,
            uuid,
            title,
            description: None,
            ingredients: Vec::new(),
            instructions: Vec::new(),
            prep_time_minutes: None,
            cook_time_minutes: None,
            servings: None,
            tags: Vec::new(),
        }
    }

    /// Returns the total time in minutes (prep + cook)
    /// Returns None if either time is not set, or if the sum would overflow
    pub fn total_time_minutes(&self) -> Option<u32> {
        match (self.prep_time_minutes, self.cook_time_minutes) {
            (Some(prep), Some(cook)) => prep.checked_add(cook),
            (Some(prep), None) => Some(prep),
            (None, Some(cook)) => Some(cook),
            (None, None) => None,
        }
    }

    /// Checks if the recipe has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_recipe() {
        let recipe = Recipe::new("recipe1".to_string(), "Pasta Carbonara".to_string());
        assert_eq!(recipe.id, "recipe1");
        assert_eq!(recipe.title, "Pasta Carbonara");
        assert_eq!(recipe.description, None);
        assert_eq!(recipe.ingredients.len(), 0);
        assert_eq!(recipe.instructions.len(), 0);
        assert_eq!(recipe.tags.len(), 0);
    }

    #[test]
    fn test_recipe_with_all_fields() {
        let mut recipe = Recipe::new("recipe2".to_string(), "Chocolate Cake".to_string());
        recipe.description = Some("A delicious chocolate cake".to_string());
        recipe.ingredients = vec![
            "2 cups flour".to_string(),
            "1 cup sugar".to_string(),
            "1/2 cup cocoa powder".to_string(),
        ];
        recipe.instructions = vec![
            "Mix dry ingredients".to_string(),
            "Add wet ingredients".to_string(),
            "Bake at 350F for 30 minutes".to_string(),
        ];
        recipe.prep_time_minutes = Some(20);
        recipe.cook_time_minutes = Some(30);
        recipe.servings = Some(8);
        recipe.tags = vec!["dessert".to_string(), "chocolate".to_string()];

        assert_eq!(recipe.id, "recipe2");
        assert_eq!(recipe.title, "Chocolate Cake");
        assert_eq!(
            recipe.description,
            Some("A delicious chocolate cake".to_string())
        );
        assert_eq!(recipe.ingredients.len(), 3);
        assert_eq!(recipe.instructions.len(), 3);
        assert_eq!(recipe.prep_time_minutes, Some(20));
        assert_eq!(recipe.cook_time_minutes, Some(30));
        assert_eq!(recipe.servings, Some(8));
        assert_eq!(recipe.tags.len(), 2);
    }

    #[test]
    fn test_total_time_minutes_with_both() {
        let mut recipe = Recipe::new("recipe3".to_string(), "Quick Salad".to_string());
        recipe.prep_time_minutes = Some(10);
        recipe.cook_time_minutes = Some(5);
        assert_eq!(recipe.total_time_minutes(), Some(15));
    }

    #[test]
    fn test_total_time_minutes_prep_only() {
        let mut recipe = Recipe::new("recipe4".to_string(), "Fresh Salad".to_string());
        recipe.prep_time_minutes = Some(10);
        assert_eq!(recipe.total_time_minutes(), Some(10));
    }

    #[test]
    fn test_total_time_minutes_cook_only() {
        let mut recipe = Recipe::new("recipe5".to_string(), "Boiled Eggs".to_string());
        recipe.cook_time_minutes = Some(12);
        assert_eq!(recipe.total_time_minutes(), Some(12));
    }

    #[test]
    fn test_total_time_minutes_none() {
        let recipe = Recipe::new("recipe6".to_string(), "Mystery Dish".to_string());
        assert_eq!(recipe.total_time_minutes(), None);
    }

    #[test]
    fn test_has_tag_case_insensitive() {
        let mut recipe = Recipe::new("recipe7".to_string(), "Veggie Burger".to_string());
        recipe.tags = vec!["Vegetarian".to_string(), "Quick".to_string()];

        assert!(recipe.has_tag("vegetarian"));
        assert!(recipe.has_tag("VEGETARIAN"));
        assert!(recipe.has_tag("Vegetarian"));
        assert!(recipe.has_tag("quick"));
        assert!(!recipe.has_tag("meat"));
    }

    #[test]
    fn test_has_tag_empty() {
        let recipe = Recipe::new("recipe8".to_string(), "Plain Rice".to_string());
        assert!(!recipe.has_tag("any_tag"));
    }

    #[test]
    fn test_recipe_clone() {
        let recipe = Recipe::new("recipe9".to_string(), "Clone Test".to_string());
        let cloned = recipe.clone();
        assert_eq!(recipe, cloned);
    }

    #[test]
    fn test_recipe_equality() {
        let uuid = Uuid::new_v4();
        let recipe1 =
            Recipe::new_with_uuid("recipe10".to_string(), uuid, "Same Recipe".to_string());
        let recipe2 =
            Recipe::new_with_uuid("recipe10".to_string(), uuid, "Same Recipe".to_string());
        assert_eq!(recipe1, recipe2);

        // Test inequality with different UUIDs
        let uuid2 = Uuid::new_v4();
        let recipe3 =
            Recipe::new_with_uuid("recipe10".to_string(), uuid2, "Same Recipe".to_string());
        assert_ne!(recipe1, recipe3);
    }

    #[test]
    fn test_total_time_minutes_overflow() {
        let mut recipe = Recipe::new("recipe11".to_string(), "Long Recipe".to_string());
        recipe.prep_time_minutes = Some(u32::MAX);
        recipe.cook_time_minutes = Some(1);
        // Should return None on overflow instead of panicking
        assert_eq!(recipe.total_time_minutes(), None);
    }

    // Tests for RecipeError
    #[test]
    fn test_recipe_error_display() {
        let error = RecipeError::NotFound("recipe123".to_string());
        assert_eq!(error.to_string(), "Recipe not found: recipe123");

        let error = RecipeError::StorageError("disk full".to_string());
        assert_eq!(error.to_string(), "Storage error: disk full");

        let error = RecipeError::InvalidData("missing title".to_string());
        assert_eq!(error.to_string(), "Invalid data: missing title");

        let error = RecipeError::AlreadyExists("recipe456".to_string());
        assert_eq!(error.to_string(), "Recipe already exists: recipe456");
    }

    #[test]
    fn test_recipe_error_equality() {
        let error1 = RecipeError::NotFound("test".to_string());
        let error2 = RecipeError::NotFound("test".to_string());
        let error3 = RecipeError::NotFound("other".to_string());

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    // Mock implementation of RecipeReader for testing
    struct MockRecipeReader {
        recipes: Vec<Recipe>,
    }

    impl RecipeReader for MockRecipeReader {
        fn get_by_id(&self, id: &str) -> RecipeResult<Recipe> {
            self.recipes
                .iter()
                .find(|r| r.id == id)
                .cloned()
                .ok_or_else(|| RecipeError::NotFound(format!("Recipe with id '{}' not found", id)))
        }

        fn get_by_uuid(&self, uuid: &Uuid) -> RecipeResult<Recipe> {
            self.recipes
                .iter()
                .find(|r| r.uuid == *uuid)
                .cloned()
                .ok_or_else(|| {
                    RecipeError::NotFound(format!("Recipe with uuid '{}' not found", uuid))
                })
        }

        fn get_by_day(&self, day: u32) -> RecipeResult<Recipe> {
            if !(1..=366).contains(&day) {
                return Err(RecipeError::InvalidData(format!(
                    "Day must be between 1 and 366, got {}",
                    day
                )));
            }
            // For testing, map day to recipe ID
            let id = format!("day-{}", day);
            self.get_by_id(&id)
        }

        fn get_all(&self) -> RecipeResult<Vec<Recipe>> {
            Ok(self.recipes.clone())
        }

        fn exists(&self, id: &str) -> bool {
            self.recipes.iter().any(|r| r.id == id)
        }
    }

    #[test]
    fn test_recipe_reader_get_by_id() {
        let recipe1 = Recipe::new("recipe1".to_string(), "Test Recipe 1".to_string());
        let recipe2 = Recipe::new("recipe2".to_string(), "Test Recipe 2".to_string());
        let reader = MockRecipeReader {
            recipes: vec![recipe1.clone(), recipe2],
        };

        let result = reader.get_by_id("recipe1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), recipe1);

        let result = reader.get_by_id("nonexistent");
        assert!(result.is_err());
        match result {
            Err(RecipeError::NotFound(msg)) => {
                assert!(msg.contains("nonexistent"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_recipe_reader_get_by_uuid() {
        let recipe1 = Recipe::new("recipe1".to_string(), "Test Recipe 1".to_string());
        let recipe2 = Recipe::new("recipe2".to_string(), "Test Recipe 2".to_string());
        let uuid1 = recipe1.uuid;
        let uuid2 = recipe2.uuid;
        let reader = MockRecipeReader {
            recipes: vec![recipe1.clone(), recipe2],
        };

        let result = reader.get_by_uuid(&uuid1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), recipe1);

        let nonexistent_uuid = Uuid::new_v4();
        let result = reader.get_by_uuid(&nonexistent_uuid);
        assert!(result.is_err());
        match result {
            Err(RecipeError::NotFound(msg)) => {
                assert!(msg.contains(&nonexistent_uuid.to_string()));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_recipe_reader_get_by_day() {
        let recipe1 = Recipe::new("day-1".to_string(), "Day 1 Recipe".to_string());
        let recipe100 = Recipe::new("day-100".to_string(), "Day 100 Recipe".to_string());
        let reader = MockRecipeReader {
            recipes: vec![recipe1.clone(), recipe100.clone()],
        };

        let result = reader.get_by_day(1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), recipe1);

        let result = reader.get_by_day(100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), recipe100);

        // Test invalid day
        let result = reader.get_by_day(0);
        assert!(result.is_err());
        match result {
            Err(RecipeError::InvalidData(_)) => {}
            _ => panic!("Expected InvalidData error for day 0"),
        }

        let result = reader.get_by_day(367);
        assert!(result.is_err());
        match result {
            Err(RecipeError::InvalidData(_)) => {}
            _ => panic!("Expected InvalidData error for day 367"),
        }
    }

    #[test]
    fn test_recipe_reader_get_all() {
        let recipe1 = Recipe::new("recipe1".to_string(), "Test Recipe 1".to_string());
        let recipe2 = Recipe::new("recipe2".to_string(), "Test Recipe 2".to_string());
        let reader = MockRecipeReader {
            recipes: vec![recipe1.clone(), recipe2.clone()],
        };

        let result = reader.get_all();
        assert!(result.is_ok());
        let all_recipes = result.unwrap();
        assert_eq!(all_recipes.len(), 2);
        assert_eq!(all_recipes[0], recipe1);
        assert_eq!(all_recipes[1], recipe2);
    }

    #[test]
    fn test_recipe_reader_exists() {
        let recipe1 = Recipe::new("recipe1".to_string(), "Test Recipe 1".to_string());
        let reader = MockRecipeReader {
            recipes: vec![recipe1],
        };

        assert!(reader.exists("recipe1"));
        assert!(!reader.exists("nonexistent"));
    }

    #[test]
    fn test_recipe_reader_get_by_tag() {
        let mut recipe1 = Recipe::new("recipe1".to_string(), "Veg Recipe".to_string());
        recipe1.tags = vec!["vegetarian".to_string(), "quick".to_string()];

        let mut recipe2 = Recipe::new("recipe2".to_string(), "Meat Recipe".to_string());
        recipe2.tags = vec!["meat".to_string()];

        let mut recipe3 = Recipe::new("recipe3".to_string(), "Quick Snack".to_string());
        recipe3.tags = vec!["quick".to_string(), "snack".to_string()];

        let reader = MockRecipeReader {
            recipes: vec![recipe1.clone(), recipe2, recipe3.clone()],
        };

        let result = reader.get_by_tag("quick");
        assert!(result.is_ok());
        let quick_recipes = result.unwrap();
        assert_eq!(quick_recipes.len(), 2);
        assert!(quick_recipes.contains(&recipe1));
        assert!(quick_recipes.contains(&recipe3));

        let result = reader.get_by_tag("vegetarian");
        assert!(result.is_ok());
        let veg_recipes = result.unwrap();
        assert_eq!(veg_recipes.len(), 1);
        assert_eq!(veg_recipes[0], recipe1);

        let result = reader.get_by_tag("nonexistent");
        assert!(result.is_ok());
        let empty_recipes = result.unwrap();
        assert_eq!(empty_recipes.len(), 0);
    }

    // Mock implementation of RecipeWriter for testing
    struct MockRecipeWriter {
        recipes: Vec<Recipe>,
    }

    impl RecipeWriter for MockRecipeWriter {
        fn create(&mut self, recipe: Recipe) -> RecipeResult<()> {
            if self.recipes.iter().any(|r| r.id == recipe.id) {
                return Err(RecipeError::AlreadyExists(format!(
                    "Recipe with id '{}' already exists",
                    recipe.id
                )));
            }
            self.recipes.push(recipe);
            Ok(())
        }

        fn update(&mut self, recipe: Recipe) -> RecipeResult<()> {
            let pos = self
                .recipes
                .iter()
                .position(|r| r.id == recipe.id)
                .ok_or_else(|| {
                    RecipeError::NotFound(format!("Recipe with id '{}' not found", recipe.id))
                })?;
            self.recipes[pos] = recipe;
            Ok(())
        }

        fn delete(&mut self, id: &str) -> RecipeResult<()> {
            let pos = self
                .recipes
                .iter()
                .position(|r| r.id == id)
                .ok_or_else(|| {
                    RecipeError::NotFound(format!("Recipe with id '{}' not found", id))
                })?;
            self.recipes.remove(pos);
            Ok(())
        }

        fn save(&mut self, recipe: Recipe) -> RecipeResult<()> {
            // Efficient implementation that checks existence first
            if let Some(pos) = self.recipes.iter().position(|r| r.id == recipe.id) {
                self.recipes[pos] = recipe;
            } else {
                self.recipes.push(recipe);
            }
            Ok(())
        }
    }

    #[test]
    fn test_recipe_writer_create() {
        let mut writer = MockRecipeWriter {
            recipes: Vec::new(),
        };

        let recipe1 = Recipe::new("recipe1".to_string(), "New Recipe".to_string());
        let result = writer.create(recipe1.clone());
        assert!(result.is_ok());
        assert_eq!(writer.recipes.len(), 1);
        assert_eq!(writer.recipes[0], recipe1);

        // Try to create duplicate
        let result = writer.create(recipe1.clone());
        assert!(result.is_err());
        match result {
            Err(RecipeError::AlreadyExists(msg)) => {
                assert!(msg.contains("recipe1"));
            }
            _ => panic!("Expected AlreadyExists error"),
        }
    }

    #[test]
    fn test_recipe_writer_update() {
        let recipe1 = Recipe::new("recipe1".to_string(), "Original Recipe".to_string());
        let mut writer = MockRecipeWriter {
            recipes: vec![recipe1],
        };

        let mut updated = Recipe::new("recipe1".to_string(), "Updated Recipe".to_string());
        updated.description = Some("New description".to_string());

        let result = writer.update(updated.clone());
        assert!(result.is_ok());
        assert_eq!(writer.recipes[0], updated);

        // Try to update nonexistent
        let nonexistent = Recipe::new("nonexistent".to_string(), "Does Not Exist".to_string());
        let result = writer.update(nonexistent);
        assert!(result.is_err());
        match result {
            Err(RecipeError::NotFound(msg)) => {
                assert!(msg.contains("nonexistent"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_recipe_writer_delete() {
        let recipe1 = Recipe::new("recipe1".to_string(), "Recipe 1".to_string());
        let recipe2 = Recipe::new("recipe2".to_string(), "Recipe 2".to_string());
        let mut writer = MockRecipeWriter {
            recipes: vec![recipe1, recipe2.clone()],
        };

        let result = writer.delete("recipe1");
        assert!(result.is_ok());
        assert_eq!(writer.recipes.len(), 1);
        assert_eq!(writer.recipes[0], recipe2);

        // Try to delete nonexistent
        let result = writer.delete("nonexistent");
        assert!(result.is_err());
        match result {
            Err(RecipeError::NotFound(msg)) => {
                assert!(msg.contains("nonexistent"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_recipe_writer_save_create() {
        let mut writer = MockRecipeWriter {
            recipes: Vec::new(),
        };

        let recipe1 = Recipe::new("recipe1".to_string(), "New Recipe".to_string());
        let result = writer.save(recipe1.clone());
        assert!(result.is_ok());
        assert_eq!(writer.recipes.len(), 1);
        assert_eq!(writer.recipes[0], recipe1);
    }

    #[test]
    fn test_recipe_writer_save_update() {
        let recipe1 = Recipe::new("recipe1".to_string(), "Original Recipe".to_string());
        let mut writer = MockRecipeWriter {
            recipes: vec![recipe1],
        };

        let mut updated = Recipe::new("recipe1".to_string(), "Updated Recipe".to_string());
        updated.description = Some("Updated description".to_string());

        let result = writer.save(updated.clone());
        assert!(result.is_ok());
        assert_eq!(writer.recipes.len(), 1);
        assert_eq!(writer.recipes[0], updated);
    }
}

/// Errors that can occur when working with plans
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanError {
    /// Plan was not found
    NotFound(String),
    /// I/O or storage error occurred
    StorageError(String),
    /// Plan data is invalid or malformed
    InvalidData(String),
    /// Plan already exists (for create operations)
    AlreadyExists(String),
}

impl fmt::Display for PlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlanError::NotFound(msg) => write!(f, "Plan not found: {}", msg),
            PlanError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            PlanError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            PlanError::AlreadyExists(msg) => write!(f, "Plan already exists: {}", msg),
        }
    }
}

impl std::error::Error for PlanError {}

/// Result type for plan operations
pub type PlanResult<T> = Result<T, PlanError>;

/// Represents a weekly meal plan with associated recipes.
///
/// Plans use ISO 8601 week numbering (weeks 1-53, starting Monday).
/// Each plan contains 7 recipe UUIDs representing the recipes
/// for Monday through Sunday of that week.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// ISO 8601 week number (1-53)
    pub week: u32,
    /// Recipe UUIDs for this week's plan, ordered Monday through Sunday.
    /// Must contain exactly 7 values.
    pub recipe_uuids: Vec<Uuid>,
    /// Legacy: Day-of-year values (1-365/366) for recipes in this week's plan.
    /// Deprecated - use recipe_uuids instead. Will be removed in future version.
    #[deprecated(note = "Use recipe_uuids instead")]
    pub recipe_days: Option<Vec<u32>>,
}

impl Plan {
    /// Creates a new plan with a week number and 7 recipe UUID references.
    ///
    /// # Arguments
    /// * `week` - ISO 8601 week number (1-53)
    /// * `recipe_uuids` - Vector of 7 recipe UUIDs for Mon-Sun
    ///
    /// # Panics
    /// Panics if recipe_uuids doesn't contain exactly 7 values.
    /// Use `new_checked` for a non-panicking alternative.
    pub fn new(week: u32, recipe_uuids: Vec<Uuid>) -> Self {
        assert_eq!(
            recipe_uuids.len(),
            7,
            "Plan must have exactly 7 recipe UUIDs, got {}",
            recipe_uuids.len()
        );
        #[allow(deprecated)]
        Self {
            week,
            recipe_uuids,
            recipe_days: None,
        }
    }

    /// Creates a new plan with legacy day-of-year references (deprecated).
    ///
    /// This method is provided for backward compatibility during migration.
    /// Use `new()` with recipe UUIDs for new code.
    #[deprecated(note = "Use new() with recipe UUIDs instead")]
    #[allow(deprecated)]
    pub fn new_with_days(week: u32, recipe_days: Vec<u32>) -> Self {
        assert_eq!(
            recipe_days.len(),
            7,
            "Plan must have exactly 7 recipe days, got {}",
            recipe_days.len()
        );
        Self {
            week,
            recipe_uuids: vec![Uuid::nil(); 7], // Placeholder UUIDs
            recipe_days: Some(recipe_days),
        }
    }

    /// Creates a new plan with validation.
    ///
    /// Returns an error if:
    /// - Week number is not in range 1-53
    /// - recipe_uuids doesn't contain exactly 7 values
    pub fn new_checked(week: u32, recipe_uuids: Vec<Uuid>) -> PlanResult<Self> {
        if !(1..=53).contains(&week) {
            return Err(PlanError::InvalidData(format!(
                "Week must be between 1 and 53, got {}",
                week
            )));
        }

        if recipe_uuids.len() != 7 {
            return Err(PlanError::InvalidData(format!(
                "Plan must have exactly 7 recipe UUIDs, got {}",
                recipe_uuids.len()
            )));
        }

        #[allow(deprecated)]
        Ok(Self {
            week,
            recipe_uuids,
            recipe_days: None,
        })
    }

    /// Get the recipe UUID for a specific weekday (0=Monday, 6=Sunday)
    pub fn get_uuid_for_weekday(&self, weekday: usize) -> Option<Uuid> {
        self.recipe_uuids.get(weekday).copied()
    }

    /// Legacy: Get the recipe day for a specific weekday (deprecated)
    #[deprecated(note = "Use get_uuid_for_weekday instead")]
    #[allow(deprecated)]
    pub fn get_day_for_weekday(&self, weekday: usize) -> Option<u32> {
        self.recipe_days
            .as_ref()
            .and_then(|days| days.get(weekday).copied())
    }
}

/// Trait for reading plan information from a data source
pub trait PlanReader {
    /// Get a plan by its ISO 8601 week number (1-53)
    fn get_by_week(&self, week: u32) -> PlanResult<Plan>;

    /// Get all plans
    fn get_all(&self) -> PlanResult<Vec<Plan>>;

    /// Check if a plan exists for a given week
    fn exists(&self, week: u32) -> bool;
}

/// Trait for writing plan information to a data source
pub trait PlanWriter {
    /// Create a new plan
    /// Returns an error if a plan for that week already exists
    fn create(&mut self, plan: Plan) -> PlanResult<()>;

    /// Update an existing plan
    /// Returns an error if the plan doesn't exist
    fn update(&mut self, plan: Plan) -> PlanResult<()>;

    /// Delete a plan by week number
    /// Returns an error if the plan doesn't exist
    fn delete(&mut self, week: u32) -> PlanResult<()>;

    /// Create or update a plan (upsert)
    fn save(&mut self, plan: Plan) -> PlanResult<()>;
}

#[cfg(test)]
mod plan_tests {
    use super::*;

    // Tests for PlanError
    #[test]
    fn test_plan_error_display() {
        let error = PlanError::NotFound("week 5".to_string());
        assert_eq!(error.to_string(), "Plan not found: week 5");

        let error = PlanError::StorageError("disk full".to_string());
        assert_eq!(error.to_string(), "Storage error: disk full");

        let error = PlanError::InvalidData("invalid week".to_string());
        assert_eq!(error.to_string(), "Invalid data: invalid week");

        let error = PlanError::AlreadyExists("week 10".to_string());
        assert_eq!(error.to_string(), "Plan already exists: week 10");
    }

    #[test]
    fn test_plan_error_equality() {
        let error1 = PlanError::NotFound("test".to_string());
        let error2 = PlanError::NotFound("test".to_string());
        let error3 = PlanError::NotFound("other".to_string());

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    // Tests for Plan struct
    #[test]
    fn test_plan_new() {
        let uuids = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let plan = Plan::new(1, uuids.clone());
        assert_eq!(plan.week, 1);
        assert_eq!(plan.recipe_uuids, uuids);
    }

    #[test]
    #[should_panic(expected = "Plan must have exactly 7 recipe UUIDs")]
    fn test_plan_new_wrong_length_panics() {
        let uuids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()]; // Only 3 UUIDs
        Plan::new(1, uuids);
    }

    #[test]
    fn test_plan_new_checked_valid() {
        let uuids = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let result = Plan::new_checked(1, uuids.clone());
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert_eq!(plan.week, 1);
        assert_eq!(plan.recipe_uuids, uuids);
    }

    #[test]
    fn test_plan_new_checked_invalid_week_zero() {
        let uuids = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let result = Plan::new_checked(0, uuids);
        assert!(result.is_err());
        match result {
            Err(PlanError::InvalidData(msg)) => {
                assert!(msg.contains("Week must be between 1 and 53"));
            }
            _ => panic!("Expected InvalidData error"),
        }
    }

    #[test]
    fn test_plan_new_checked_invalid_week_too_high() {
        let uuids = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let result = Plan::new_checked(54, uuids);
        assert!(result.is_err());
        match result {
            Err(PlanError::InvalidData(msg)) => {
                assert!(msg.contains("Week must be between 1 and 53"));
            }
            _ => panic!("Expected InvalidData error"),
        }
    }

    #[test]
    fn test_plan_new_checked_wrong_length() {
        let uuids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()]; // Only 3 UUIDs
        let result = Plan::new_checked(1, uuids);
        assert!(result.is_err());
        match result {
            Err(PlanError::InvalidData(msg)) => {
                assert!(msg.contains("Plan must have exactly 7 recipe UUIDs"));
            }
            _ => panic!("Expected InvalidData error"),
        }
    }

    #[test]
    #[allow(deprecated)]
    fn test_plan_new_with_days_legacy() {
        let days = vec![1, 2, 3, 4, 5, 6, 7];
        let plan = Plan::new_with_days(1, days.clone());
        assert_eq!(plan.week, 1);
        assert_eq!(plan.recipe_days, Some(days));
    }

    #[test]
    fn test_plan_get_uuid_for_weekday() {
        let uuids = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let plan = Plan::new(2, uuids.clone());

        // Monday (0)
        assert_eq!(plan.get_uuid_for_weekday(0), Some(uuids[0]));
        // Tuesday (1)
        assert_eq!(plan.get_uuid_for_weekday(1), Some(uuids[1]));
        // Sunday (6)
        assert_eq!(plan.get_uuid_for_weekday(6), Some(uuids[6]));
        // Invalid weekday
        assert_eq!(plan.get_uuid_for_weekday(7), None);
    }

    #[test]
    #[allow(deprecated)]
    fn test_plan_get_day_for_weekday_legacy() {
        let days = vec![10, 11, 12, 13, 14, 15, 16];
        let plan = Plan::new_with_days(2, days.clone());

        // Monday (0)
        assert_eq!(plan.get_day_for_weekday(0), Some(10));
        // Tuesday (1)
        assert_eq!(plan.get_day_for_weekday(1), Some(11));
        // Sunday (6)
        assert_eq!(plan.get_day_for_weekday(6), Some(16));
        // Invalid weekday
        assert_eq!(plan.get_day_for_weekday(7), None);
    }

    #[test]
    fn test_plan_clone() {
        let uuids = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let plan = Plan::new(1, uuids);
        let cloned = plan.clone();
        assert_eq!(plan, cloned);
    }

    #[test]
    fn test_plan_equality() {
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();
        let uuid3 = Uuid::new_v4();
        let uuid4 = Uuid::new_v4();
        let uuid5 = Uuid::new_v4();
        let uuid6 = Uuid::new_v4();
        let uuid7 = Uuid::new_v4();

        let plan1 = Plan::new(1, vec![uuid1, uuid2, uuid3, uuid4, uuid5, uuid6, uuid7]);
        let plan2 = Plan::new(1, vec![uuid1, uuid2, uuid3, uuid4, uuid5, uuid6, uuid7]);
        assert_eq!(plan1, plan2);

        let uuid8 = Uuid::new_v4();
        let plan3 = Plan::new(1, vec![uuid8, uuid2, uuid3, uuid4, uuid5, uuid6, uuid7]);
        assert_ne!(plan1, plan3);
    }

    // Mock implementation of PlanReader for testing
    struct MockPlanReader {
        plans: Vec<Plan>,
    }

    impl PlanReader for MockPlanReader {
        fn get_by_week(&self, week: u32) -> PlanResult<Plan> {
            self.plans
                .iter()
                .find(|p| p.week == week)
                .cloned()
                .ok_or_else(|| PlanError::NotFound(format!("Plan for week {} not found", week)))
        }

        fn get_all(&self) -> PlanResult<Vec<Plan>> {
            Ok(self.plans.clone())
        }

        fn exists(&self, week: u32) -> bool {
            self.plans.iter().any(|p| p.week == week)
        }
    }

    #[test]
    fn test_plan_reader_get_by_week() {
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
        let plan1 = Plan::new(1, uuids1);
        let plan2 = Plan::new(2, uuids2);
        let reader = MockPlanReader {
            plans: vec![plan1.clone(), plan2],
        };

        let result = reader.get_by_week(1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), plan1);

        let result = reader.get_by_week(99);
        assert!(result.is_err());
        match result {
            Err(PlanError::NotFound(msg)) => {
                assert!(msg.contains("week 99"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_plan_reader_get_all() {
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
        let plan1 = Plan::new(1, uuids1);
        let plan2 = Plan::new(2, uuids2);
        let reader = MockPlanReader {
            plans: vec![plan1.clone(), plan2.clone()],
        };

        let result = reader.get_all();
        assert!(result.is_ok());
        let all_plans = result.unwrap();
        assert_eq!(all_plans.len(), 2);
        assert_eq!(all_plans[0], plan1);
        assert_eq!(all_plans[1], plan2);
    }

    #[test]
    fn test_plan_reader_exists() {
        let uuids = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let plan1 = Plan::new(1, uuids);
        let reader = MockPlanReader { plans: vec![plan1] };

        assert!(reader.exists(1));
        assert!(!reader.exists(99));
    }

    // Mock implementation of PlanWriter for testing
    struct MockPlanWriter {
        plans: Vec<Plan>,
    }

    impl PlanWriter for MockPlanWriter {
        fn create(&mut self, plan: Plan) -> PlanResult<()> {
            if self.plans.iter().any(|p| p.week == plan.week) {
                return Err(PlanError::AlreadyExists(format!(
                    "Plan for week {} already exists",
                    plan.week
                )));
            }
            self.plans.push(plan);
            Ok(())
        }

        fn update(&mut self, plan: Plan) -> PlanResult<()> {
            let pos = self
                .plans
                .iter()
                .position(|p| p.week == plan.week)
                .ok_or_else(|| {
                    PlanError::NotFound(format!("Plan for week {} not found", plan.week))
                })?;
            self.plans[pos] = plan;
            Ok(())
        }

        fn delete(&mut self, week: u32) -> PlanResult<()> {
            let pos = self
                .plans
                .iter()
                .position(|p| p.week == week)
                .ok_or_else(|| PlanError::NotFound(format!("Plan for week {} not found", week)))?;
            self.plans.remove(pos);
            Ok(())
        }

        fn save(&mut self, plan: Plan) -> PlanResult<()> {
            if let Some(pos) = self.plans.iter().position(|p| p.week == plan.week) {
                self.plans[pos] = plan;
            } else {
                self.plans.push(plan);
            }
            Ok(())
        }
    }

    #[test]
    fn test_plan_writer_create() {
        let mut writer = MockPlanWriter { plans: Vec::new() };

        let uuids = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let plan1 = Plan::new(1, uuids);
        let result = writer.create(plan1.clone());
        assert!(result.is_ok());
        assert_eq!(writer.plans.len(), 1);
        assert_eq!(writer.plans[0], plan1);

        // Try to create duplicate
        let result = writer.create(plan1.clone());
        assert!(result.is_err());
        match result {
            Err(PlanError::AlreadyExists(msg)) => {
                assert!(msg.contains("week 1"));
            }
            _ => panic!("Expected AlreadyExists error"),
        }
    }

    #[test]
    fn test_plan_writer_update() {
        let uuids1 = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let plan1 = Plan::new(1, uuids1);
        let mut writer = MockPlanWriter { plans: vec![plan1] };

        let uuids2 = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let updated = Plan::new(1, uuids2);
        let result = writer.update(updated.clone());
        assert!(result.is_ok());
        assert_eq!(writer.plans[0], updated);

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
        let nonexistent = Plan::new(99, uuids3);
        let result = writer.update(nonexistent);
        assert!(result.is_err());
        match result {
            Err(PlanError::NotFound(msg)) => {
                assert!(msg.contains("week 99"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_plan_writer_delete() {
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
        let plan1 = Plan::new(1, uuids1);
        let plan2 = Plan::new(2, uuids2);
        let mut writer = MockPlanWriter {
            plans: vec![plan1, plan2.clone()],
        };

        let result = writer.delete(1);
        assert!(result.is_ok());
        assert_eq!(writer.plans.len(), 1);
        assert_eq!(writer.plans[0], plan2);

        // Try to delete nonexistent
        let result = writer.delete(99);
        assert!(result.is_err());
        match result {
            Err(PlanError::NotFound(msg)) => {
                assert!(msg.contains("week 99"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_plan_writer_save_create() {
        let mut writer = MockPlanWriter { plans: Vec::new() };

        let uuids = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let plan1 = Plan::new(1, uuids);
        let result = writer.save(plan1.clone());
        assert!(result.is_ok());
        assert_eq!(writer.plans.len(), 1);
        assert_eq!(writer.plans[0], plan1);
    }

    #[test]
    fn test_plan_writer_save_update() {
        let uuids1 = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let plan1 = Plan::new(1, uuids1);
        let mut writer = MockPlanWriter { plans: vec![plan1] };

        let uuids2 = vec![
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let updated = Plan::new(1, uuids2);
        let result = writer.save(updated.clone());
        assert!(result.is_ok());
        assert_eq!(writer.plans.len(), 1);
        assert_eq!(writer.plans[0], updated);
    }
}
