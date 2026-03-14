use rand::Rng;
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::card::Card;

/// A deck of cards held by a player.
#[derive(Debug, Default)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn remaining(&self) -> usize {
        self.cards.len()
    }

    /// Shuffles the deck in place using the thread-local random number generator.
    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut thread_rng());
    }

    /// Shuffles the deck in place using the provided random number generator.
    ///
    /// Prefer this variant in tests where deterministic output is required.
    #[allow(dead_code)]
    pub fn shuffle_with(&mut self, rng: &mut impl Rng) {
        self.cards.shuffle(rng);
    }

    /// Returns a slice of all cards currently in the deck (top-to-bottom order).
    pub fn cards(&self) -> &[Card] {
        &self.cards
    }
}

/// Builder for constructing a [`Deck`] from a set of chosen cards.
///
/// Enforces an optional per-card copy limit so that no single card name
/// appears more than `max_copies` times in the finished deck.
#[derive(Debug, Default)]
pub struct DeckBuilder {
    cards: Vec<Card>,
    max_copies: Option<usize>,
}

impl DeckBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of copies of any single card (by name) allowed
    /// in the deck.  Cards that would exceed this limit are silently ignored.
    pub fn with_max_copies(mut self, max: usize) -> Self {
        self.max_copies = Some(max);
        self
    }

    /// Attempt to add a card to the in-progress deck.
    ///
    /// Returns `true` if the card was added, or `false` if it was rejected
    /// because the copy limit would have been exceeded.
    pub fn add_card(&mut self, card: Card) -> bool {
        if let Some(max) = self.max_copies {
            let count = self.cards.iter().filter(|c| c.name == card.name).count();
            if count >= max {
                return false;
            }
        }
        self.cards.push(card);
        true
    }

    /// Consume the builder and return the finished [`Deck`].
    pub fn build(self) -> Deck {
        Deck { cards: self.cards }
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    use super::*;

    // ── Deck ──────────────────────────────────────────────────────────────────

    #[test]
    fn deck_starts_empty() {
        let deck = Deck::new();
        assert_eq!(deck.remaining(), 0);
    }

    #[test]
    fn deck_add_and_draw_card() {
        let mut deck = Deck::new();
        deck.add_card(Card::new("Ace", 1));
        assert_eq!(deck.remaining(), 1);
        let drawn = deck.draw();
        assert!(drawn.is_some());
        assert_eq!(deck.remaining(), 0);
    }

    #[test]
    fn deck_draw_empty_returns_none() {
        let mut deck = Deck::new();
        assert!(deck.draw().is_none());
    }

    #[test]
    fn deck_cards_returns_all_cards() {
        let mut deck = Deck::new();
        deck.add_card(Card::new("Ace", 14));
        deck.add_card(Card::new("King", 13));
        assert_eq!(deck.cards().len(), 2);
    }

    #[test]
    fn deck_shuffle_preserves_card_count() {
        let mut deck = Deck::new();
        for i in 1..=10u32 {
            deck.add_card(Card::new(format!("Card {i}"), i));
        }
        let mut rng = SmallRng::seed_from_u64(42);
        deck.shuffle_with(&mut rng);
        assert_eq!(deck.remaining(), 10);
    }

    #[test]
    fn deck_shuffle_changes_order() {
        let mut deck = Deck::new();
        for i in 1..=10u32 {
            deck.add_card(Card::new(format!("Card {i}"), i));
        }
        let original: Vec<String> = deck.cards().iter().map(|c| c.name.clone()).collect();

        let mut rng = SmallRng::seed_from_u64(42);
        deck.shuffle_with(&mut rng);
        let shuffled: Vec<String> = deck.cards().iter().map(|c| c.name.clone()).collect();

        // With 10 cards and a seeded RNG this will not be the same order.
        assert_ne!(original, shuffled);
    }

    #[test]
    fn deck_shuffle_convenience_preserves_card_count() {
        let mut deck = Deck::new();
        for i in 1..=5u32 {
            deck.add_card(Card::new(format!("Card {i}"), i));
        }
        deck.shuffle();
        assert_eq!(deck.remaining(), 5);
    }

    // ── DeckBuilder ───────────────────────────────────────────────────────────

    #[test]
    fn deck_builder_creates_deck_with_added_cards() {
        let mut builder = DeckBuilder::new();
        builder.add_card(Card::new("Ace", 14));
        builder.add_card(Card::new("King", 13));
        let deck = builder.build();
        assert_eq!(deck.remaining(), 2);
    }

    #[test]
    fn deck_builder_respects_max_copies() {
        let mut builder = DeckBuilder::new().with_max_copies(2);
        assert!(builder.add_card(Card::new("Ace", 14)));
        assert!(builder.add_card(Card::new("Ace", 14)));
        // Third copy should be rejected.
        assert!(!builder.add_card(Card::new("Ace", 14)));
        let deck = builder.build();
        assert_eq!(deck.remaining(), 2);
    }

    #[test]
    fn deck_builder_no_limit_allows_many_copies() {
        let mut builder = DeckBuilder::new();
        for _ in 0..5 {
            assert!(builder.add_card(Card::new("Fireball", 10)));
        }
        let deck = builder.build();
        assert_eq!(deck.remaining(), 5);
    }

    #[test]
    fn deck_builder_empty_build_produces_empty_deck() {
        let builder = DeckBuilder::new();
        let deck = builder.build();
        assert_eq!(deck.remaining(), 0);
    }
}
