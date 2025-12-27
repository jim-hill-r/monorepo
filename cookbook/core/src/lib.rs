use std::fmt;

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

    /// Get a recipe by day of the year (1-365)
    fn get_by_day(&self, day: u32) -> RecipeResult<Recipe>;

    /// Get all recipes
    fn get_all(&self) -> RecipeResult<Vec<Recipe>>;

    /// Check if a recipe exists by ID
    fn exists(&self, id: &str) -> bool {
        self.get_by_id(id).is_ok()
    }

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
    fn save(&mut self, recipe: Recipe) -> RecipeResult<()> {
        match self.update(recipe.clone()) {
            Ok(()) => Ok(()),
            Err(RecipeError::NotFound(_)) => self.create(recipe),
            Err(e) => Err(e),
        }
    }
}

/// Represents a recipe with all its associated information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recipe {
    /// Unique identifier for the recipe
    pub id: String,
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
    /// Creates a new recipe with required fields
    pub fn new(id: String, title: String) -> Self {
        Self {
            id,
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
        let recipe1 = Recipe::new("recipe10".to_string(), "Same Recipe".to_string());
        let recipe2 = Recipe::new("recipe10".to_string(), "Same Recipe".to_string());
        assert_eq!(recipe1, recipe2);
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

        fn get_by_day(&self, day: u32) -> RecipeResult<Recipe> {
            if !(1..=365).contains(&day) {
                return Err(RecipeError::InvalidData(format!(
                    "Day must be between 1 and 365, got {}",
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

        let result = reader.get_by_day(366);
        assert!(result.is_err());
        match result {
            Err(RecipeError::InvalidData(_)) => {}
            _ => panic!("Expected InvalidData error for day 366"),
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
