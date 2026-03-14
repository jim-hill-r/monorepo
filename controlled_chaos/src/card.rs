use bevy::prelude::*;

/// The category of a card, corresponding to the card types defined in the
/// rulebook.  Each category has an associated color used when rendering the
/// card frame.
///
/// From the rulebook:
/// - 13 Technology Cards (Blue)
/// - 13 Government Cards (Red)
/// - 13 Environment Cards (Green)
/// - 13 Economy Cards (Yellow)
/// - 13 Crisis Cards
/// - 13 Profession Cards
/// - 13 Civilian Cards
/// - 13 Society Cards
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CardCategory {
    /// Technology cards (Blue).
    #[default]
    Technology,
    /// Government cards (Red).
    Government,
    /// Environment cards (Green).
    Environment,
    /// Economy cards (Yellow).
    Economy,
    /// Crisis cards – problems that affect all players.
    Crisis,
    /// Profession cards – roles a player can take.
    Profession,
    /// Civilian cards – support for one community, require upkeep.
    Civilian,
    /// Society cards – support for everyone.
    Society,
}

impl CardCategory {
    /// Returns the frame color used when rendering cards of this category.
    pub fn frame_color(&self) -> Color {
        match self {
            CardCategory::Technology => Color::srgb(0.15, 0.35, 0.75),
            CardCategory::Government => Color::srgb(0.65, 0.15, 0.15),
            CardCategory::Environment => Color::srgb(0.15, 0.55, 0.25),
            CardCategory::Economy => Color::srgb(0.65, 0.60, 0.10),
            CardCategory::Crisis => Color::srgb(0.50, 0.10, 0.10),
            CardCategory::Profession => Color::srgb(0.40, 0.15, 0.55),
            CardCategory::Civilian => Color::srgb(0.15, 0.50, 0.55),
            CardCategory::Society => Color::srgb(0.65, 0.35, 0.10),
        }
    }

    /// Returns a human-readable label for this category.
    pub fn label(&self) -> &'static str {
        match self {
            CardCategory::Technology => "Technology",
            CardCategory::Government => "Government",
            CardCategory::Environment => "Environment",
            CardCategory::Economy => "Economy",
            CardCategory::Crisis => "Crisis",
            CardCategory::Profession => "Profession",
            CardCategory::Civilian => "Civilian",
            CardCategory::Society => "Society",
        }
    }
}

/// A card in the game with a name, numeric value, and category.
#[derive(Component, Debug, Clone)]
pub struct Card {
    pub name: String,
    pub value: u32,
    /// The category of this card, which determines its colour and role in play.
    pub category: CardCategory,
}

impl Card {
    /// Creates a new card with the given name and value.
    ///
    /// The category defaults to [`CardCategory::Technology`].  Use
    /// [`Card::with_category`] when you need a specific category.
    pub fn new(name: impl Into<String>, value: u32) -> Self {
        Self {
            name: name.into(),
            value,
            category: CardCategory::default(),
        }
    }

    /// Creates a new card with the given name, value, and category.
    pub fn with_category(name: impl Into<String>, value: u32, category: CardCategory) -> Self {
        Self {
            name: name.into(),
            value,
            category,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_new_stores_name_and_value() {
        let card = Card::new("Ace of Spades", 14);
        assert_eq!(card.name, "Ace of Spades");
        assert_eq!(card.value, 14);
    }

    #[test]
    fn card_new_defaults_to_technology_category() {
        let card = Card::new("Ace", 14);
        assert_eq!(card.category, CardCategory::Technology);
    }

    #[test]
    fn card_with_category_stores_category() {
        let card = Card::with_category("EPA", 8, CardCategory::Government);
        assert_eq!(card.category, CardCategory::Government);
        assert_eq!(card.name, "EPA");
        assert_eq!(card.value, 8);
    }

    #[test]
    fn card_clone_produces_equal_values() {
        let card = Card::with_category("King", 13, CardCategory::Economy);
        let cloned = card.clone();
        assert_eq!(cloned.name, card.name);
        assert_eq!(cloned.value, card.value);
        assert_eq!(cloned.category, card.category);
    }

    // ── CardCategory ──────────────────────────────────────────────────────────

    #[test]
    fn card_category_default_is_technology() {
        assert_eq!(CardCategory::default(), CardCategory::Technology);
    }

    #[test]
    fn card_category_all_variants_have_distinct_labels() {
        let categories = [
            CardCategory::Technology,
            CardCategory::Government,
            CardCategory::Environment,
            CardCategory::Economy,
            CardCategory::Crisis,
            CardCategory::Profession,
            CardCategory::Civilian,
            CardCategory::Society,
        ];
        let labels: Vec<&str> = categories.iter().map(|c| c.label()).collect();
        let unique: std::collections::HashSet<&str> = labels.iter().copied().collect();
        assert_eq!(
            unique.len(),
            categories.len(),
            "each category should have a unique label"
        );
    }

    #[test]
    fn card_category_government_label() {
        assert_eq!(CardCategory::Government.label(), "Government");
    }

    #[test]
    fn card_category_crisis_frame_color_is_not_technology_color() {
        let crisis_color = CardCategory::Crisis.frame_color();
        let tech_color = CardCategory::Technology.frame_color();
        // Different categories must produce different colors.
        assert_ne!(
            format!("{crisis_color:?}"),
            format!("{tech_color:?}"),
            "Crisis and Technology should have different frame colors"
        );
    }
}
