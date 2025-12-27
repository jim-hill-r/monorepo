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
    pub fn total_time_minutes(&self) -> Option<u32> {
        match (self.prep_time_minutes, self.cook_time_minutes) {
            (Some(prep), Some(cook)) => Some(prep + cook),
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
}
