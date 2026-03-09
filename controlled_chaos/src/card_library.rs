use std::collections::HashMap;

use crate::card::Card;

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
}
