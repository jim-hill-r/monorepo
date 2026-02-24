use bevy::prelude::*;

/// A card in the game with a name and numeric value.
#[derive(Component, Debug, Clone)]
pub struct Card {
    pub name: String,
    pub value: u32,
}

impl Card {
    pub fn new(name: impl Into<String>, value: u32) -> Self {
        Self {
            name: name.into(),
            value,
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
    fn card_clone_produces_equal_values() {
        let card = Card::new("King", 13);
        let cloned = card.clone();
        assert_eq!(cloned.name, card.name);
        assert_eq!(cloned.value, card.value);
    }
}
