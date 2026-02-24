use bevy::prelude::*;

use crate::card::Card;
use crate::deck::Deck;

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

    let mut deck = Deck::new();
    deck.add_card(Card::new("Ace", 14));
    deck.add_card(Card::new("King", 13));
    deck.add_card(Card::new("Queen", 12));
    deck.add_card(Card::new("Jack", 11));

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
