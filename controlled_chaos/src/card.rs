use bevy::prelude::*;
use design_system_palette::Tokens;

use crate::theme;

/// The category of a card, corresponding to the card types defined in the
/// rulebook.  Each category is drawn in one of the design system's eight
/// categorical colours (`data-1` to `data-8`).
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
    /// Every category, in rulebook order.
    #[cfg(test)]
    pub const ALL: [CardCategory; 8] = [
        CardCategory::Technology,
        CardCategory::Government,
        CardCategory::Environment,
        CardCategory::Economy,
        CardCategory::Crisis,
        CardCategory::Profession,
        CardCategory::Civilian,
        CardCategory::Society,
    ];

    /// The category's place in [`Self::ALL`], 0 to 7: which of the design
    /// system's `data-1` to `data-8` colours it is drawn in.
    pub fn index(&self) -> usize {
        match self {
            CardCategory::Technology => 0,
            CardCategory::Government => 1,
            CardCategory::Environment => 2,
            CardCategory::Economy => 3,
            CardCategory::Crisis => 4,
            CardCategory::Profession => 5,
            CardCategory::Civilian => 6,
            CardCategory::Society => 7,
        }
    }

    /// The category's colour in the given theme: its design system data
    /// colour, used for the card frame and artwork block.
    pub fn frame_color(&self, tokens: &Tokens) -> Color {
        theme::color(tokens.data[self.index()])
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
#[derive(Component, Debug, Clone, PartialEq, Eq)]
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
    fn index_matches_position_in_all() {
        for (i, category) in CardCategory::ALL.iter().enumerate() {
            assert_eq!(category.index(), i, "{category:?}");
        }
    }

    #[test]
    fn every_category_has_its_own_colour_in_every_mode() {
        use design_system_palette::{GAMEY, Scheme};
        for scheme in [Scheme::Light, Scheme::Dark] {
            for accessible in [false, true] {
                let tokens = GAMEY.tokens(scheme, accessible);
                let colours: Vec<String> = CardCategory::ALL
                    .iter()
                    .map(|c| format!("{:?}", c.frame_color(tokens)))
                    .collect();
                let unique: std::collections::HashSet<&String> = colours.iter().collect();
                assert_eq!(unique.len(), 8, "{scheme:?} accessible={accessible}");
            }
        }
    }
}
