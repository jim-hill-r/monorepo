use std::collections::HashMap;
use std::fmt;

use crate::card::{Card, CardCategory};
use crate::deck::{Deck, DeckBuilder};

/// Maximum number of copies of any one card allowed in the action deck.
pub const MAX_COPIES_PER_CARD: usize = 2;

/// Card names that make up the default shared action deck, in build order.
pub const DEFAULT_ACTION_DECK: [&str; 18] = [
    "Ace of Spades",
    "NASA",
    "Biodome",
    "Stock Market",
    "Asteroid",
    "Engineer",
    "Suburb",
    "Space Station",
    "Solar Panel",
    "City Hall",
    "Forest",
    "Trade Route",
    "Flood",
    "Farmer",
    "Ace of Spades",
    "NASA",
    "Biodome",
    "Stock Market",
];

/// Why a deck could not be built from a list of card names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeckBuildError {
    /// The name is not registered in the library (often a typo).
    UnknownCard(String),
    /// Adding the card would exceed the per-card copy limit.
    TooManyCopies(String),
}

impl fmt::Display for DeckBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownCard(name) => write!(f, "card \"{name}\" is not in the library"),
            Self::TooManyCopies(name) => write!(
                f,
                "card \"{name}\" exceeds the limit of {MAX_COPIES_PER_CARD} copies"
            ),
        }
    }
}

impl std::error::Error for DeckBuildError {}

/// Build a deck from `names`, looking each one up in `library`.
///
/// Fails on the first name that is missing from the library or that would
/// exceed [`MAX_COPIES_PER_CARD`], so a typo can never shrink the deck
/// silently.
pub fn build_deck(library: &CardLibrary, names: &[&str]) -> Result<Deck, DeckBuildError> {
    let mut builder = DeckBuilder::new().with_max_copies(MAX_COPIES_PER_CARD);
    for &name in names {
        let card = library
            .get(name)
            .ok_or_else(|| DeckBuildError::UnknownCard(name.to_string()))?;
        if !builder.add_card(card) {
            return Err(DeckBuildError::TooManyCopies(name.to_string()));
        }
    }
    Ok(builder.build())
}

/// Build the default shared action deck (unshuffled) from `library`.
pub fn default_action_deck(library: &CardLibrary) -> Result<Deck, DeckBuildError> {
    build_deck(library, &DEFAULT_ACTION_DECK)
}

/// A catalog of cards available for players to include in their decks.
///
/// The library stores one master copy of each card keyed by name.  Players
/// browse the library and add copies of cards to their [`DeckBuilder`].
///
/// [`DeckBuilder`]: crate::deck::DeckBuilder
#[derive(Debug, Default)]
pub struct CardLibrary {
    cards: HashMap<String, Card>,
}

impl CardLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    /// The library of cards used by the base game, with categories from the
    /// rulebook.
    pub fn default_library() -> Self {
        let mut library = Self::new();
        for (name, value, category) in [
            ("Ace of Spades", 14, CardCategory::Technology),
            ("NASA", 13, CardCategory::Government),
            ("Biodome", 12, CardCategory::Environment),
            ("Stock Market", 11, CardCategory::Economy),
            ("Asteroid", 10, CardCategory::Crisis),
            ("Engineer", 9, CardCategory::Profession),
            ("Suburb", 8, CardCategory::Civilian),
            ("Space Station", 7, CardCategory::Society),
            ("Solar Panel", 6, CardCategory::Technology),
            ("City Hall", 5, CardCategory::Government),
            ("Forest", 4, CardCategory::Environment),
            ("Trade Route", 3, CardCategory::Economy),
            ("Flood", 2, CardCategory::Crisis),
            ("Farmer", 1, CardCategory::Profession),
        ] {
            library.register(Card::with_category(name, value, category));
        }
        library
    }

    /// Register a card in the library.  If a card with the same name already
    /// exists it is replaced.
    pub fn register(&mut self, card: Card) {
        self.cards.insert(card.name.clone(), card);
    }

    /// Look up a card by name, returning a clone of the master copy.
    pub fn get(&self, name: &str) -> Option<Card> {
        self.cards.get(name).cloned()
    }

    /// Returns a sorted list of all card names in the library.
    pub fn card_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.cards.keys().map(String::as_str).collect();
        names.sort_unstable();
        names
    }

    /// Returns the total number of distinct cards in the library.
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Returns `true` when the library has no cards registered.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_starts_empty() {
        let library = CardLibrary::new();
        assert!(library.is_empty());
        assert_eq!(library.len(), 0);
    }

    #[test]
    fn library_register_and_get_card() {
        let mut library = CardLibrary::new();
        library.register(Card::new("Fireball", 10));
        let card = library.get("Fireball");
        assert!(card.is_some());
        assert_eq!(card.unwrap().value, 10);
    }

    #[test]
    fn library_get_unknown_card_returns_none() {
        let library = CardLibrary::new();
        assert!(library.get("Unknown").is_none());
    }

    #[test]
    fn library_register_replaces_existing_card() {
        let mut library = CardLibrary::new();
        library.register(Card::new("Ace", 1));
        library.register(Card::new("Ace", 14));
        assert_eq!(library.get("Ace").unwrap().value, 14);
    }

    #[test]
    fn library_card_names_are_sorted() {
        let mut library = CardLibrary::new();
        library.register(Card::new("Zap", 5));
        library.register(Card::new("Ace", 1));
        library.register(Card::new("Mage", 8));
        let names = library.card_names();
        assert_eq!(names, vec!["Ace", "Mage", "Zap"]);
    }

    #[test]
    fn library_len_reflects_registered_cards() {
        let mut library = CardLibrary::new();
        library.register(Card::new("A", 1));
        library.register(Card::new("B", 2));
        assert_eq!(library.len(), 2);
    }

    #[test]
    fn default_library_has_fourteen_cards() {
        let library = CardLibrary::default_library();
        assert_eq!(library.len(), 14);
        let ace = library.get("Ace of Spades").unwrap();
        assert_eq!(ace.value, 14);
        assert_eq!(ace.category, CardCategory::Technology);
    }

    #[test]
    fn default_action_deck_has_eighteen_cards() {
        let library = CardLibrary::default_library();
        let deck = default_action_deck(&library).expect("default deck builds");
        assert_eq!(deck.remaining(), 18);
    }

    #[test]
    fn default_action_deck_respects_copy_limit() {
        let library = CardLibrary::default_library();
        let deck = default_action_deck(&library).unwrap();
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for card in deck.cards() {
            *counts.entry(card.name.as_str()).or_default() += 1;
        }
        assert!(counts.values().all(|&n| n <= MAX_COPIES_PER_CARD));
    }

    #[test]
    fn every_default_deck_name_is_in_default_library() {
        let library = CardLibrary::default_library();
        for name in DEFAULT_ACTION_DECK {
            assert!(library.get(name).is_some(), "missing card {name}");
        }
    }

    #[test]
    fn build_deck_rejects_misspelled_name() {
        let library = CardLibrary::default_library();
        let err = build_deck(&library, &["NASA", "Biodom"]).unwrap_err();
        assert_eq!(err, DeckBuildError::UnknownCard("Biodom".to_string()));
        assert_eq!(err.to_string(), "card \"Biodom\" is not in the library");
    }

    #[test]
    fn build_deck_rejects_too_many_copies() {
        let library = CardLibrary::default_library();
        let err = build_deck(&library, &["Flood", "Flood", "Flood"]).unwrap_err();
        assert_eq!(err, DeckBuildError::TooManyCopies("Flood".to_string()));
        assert_eq!(
            err.to_string(),
            "card \"Flood\" exceeds the limit of 2 copies"
        );
    }

    #[test]
    fn build_deck_with_no_names_is_empty() {
        let library = CardLibrary::default_library();
        assert_eq!(build_deck(&library, &[]).unwrap().remaining(), 0);
    }
}
