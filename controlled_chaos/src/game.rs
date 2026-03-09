use bevy::prelude::*;

use crate::card::Card;
use crate::card_library::CardLibrary;
use crate::deck::{Deck, DeckBuilder};

/// Plugin that sets up the core game systems.
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameState::default())
            .add_systems(Startup, setup)
            .add_systems(Update, log_game_state);
    }
}

/// Tracks the overall state of the game.
#[derive(Resource, Default, Debug)]
pub struct GameState {
    pub turn: u32,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // Build a card library with the available cards.
    let mut library = CardLibrary::new();
    library.register(Card::new("Ace", 14));
    library.register(Card::new("King", 13));
    library.register(Card::new("Queen", 12));
    library.register(Card::new("Jack", 11));
    library.register(Card::new("Fireball", 10));

    info!("Card library contains {} cards.", library.len());

    // Use DeckBuilder to compose a player deck (max 2 copies of any card).
    let mut builder = DeckBuilder::new().with_max_copies(2);
    for name in ["Ace", "King", "Queen", "Jack", "Ace"] {
        if let Some(card) = library.get(name) {
            builder.add_card(card);
        }
    }
    let mut deck: Deck = builder.build();

    info!("Deck initialized with {} cards.", deck.remaining());

    if let Some(card) = deck.draw() {
        info!("Drew card: {} (value: {})", card.name, card.value);
    }
}

fn log_game_state(state: Res<GameState>, mut ran: Local<bool>) {
    if !*ran {
        info!("Game started. Turn: {}", state.turn);
        *ran = true;
    }
}
