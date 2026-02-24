use crate::card::Card;

/// A deck of cards held by a player.
#[derive(Debug, Default)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn remaining(&self) -> usize {
        self.cards.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
