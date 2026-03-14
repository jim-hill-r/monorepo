use bevy::prelude::*;

use crate::card::Card;
use crate::card_library::CardLibrary;
use crate::deck::{Deck, DeckBuilder};
use crate::rules::{Player, RulesEngine};
use crate::state::AppState;

/// Plugin that sets up the core game systems.
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameState::default())
            .add_systems(OnEnter(AppState::InGame), (setup, spawn_game_ui))
            .add_systems(
                Update,
                (log_game_state, run_demo_round, update_game_ui).run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnExit(AppState::InGame), despawn_game_ui);
    }
}

/// Tracks the overall state of the game.
#[derive(Resource, Default, Debug)]
pub struct GameState {
    pub turn: u32,
}

/// Wraps the [`RulesEngine`] as a Bevy [`Resource`] so it can be accessed by
/// systems via the ECS scheduler.
#[derive(Resource)]
pub struct RulesEngineResource(pub RulesEngine);

/// Marks the root of the game UI so it can be cleaned up on exit.
#[derive(Component, Debug)]
pub struct GameUiRoot;

/// Marks the text displaying Player 1's information.
#[derive(Component, Debug)]
pub struct Player1InfoText;

/// Marks the text displaying Player 2's information.
#[derive(Component, Debug)]
pub struct Player2InfoText;

/// Marks the text displaying the current game phase.
#[derive(Component, Debug)]
pub struct GamePhaseText;

fn setup(mut commands: Commands) {
    // Build a card library with the available cards.
    let mut library = CardLibrary::new();
    library.register(Card::new("Ace", 14));
    library.register(Card::new("King", 13));
    library.register(Card::new("Queen", 12));
    library.register(Card::new("Jack", 11));
    library.register(Card::new("Fireball", 10));

    info!(
        "Card library contains {} cards: {}",
        library.len(),
        library.card_names().join(", ")
    );

    // Use DeckBuilder to compose a player deck (max 2 copies of any card).
    let mut builder = DeckBuilder::new().with_max_copies(2);
    for name in ["Ace", "King", "Queen", "Jack", "Ace"] {
        if let Some(card) = library.get(name) {
            builder.add_card(card);
        }
    }
    let mut deck: Deck = builder.build();
    deck.shuffle();

    info!("Deck initialized with {} cards.", deck.remaining());
    info!(
        "Top card after shuffle: {:?}",
        deck.cards().first().map(|c| c.name.as_str())
    );

    if let Some(card) = deck.draw() {
        info!("Drew card: {} (value: {})", card.name, card.value);
    }

    // Initialise the rules engine with two players and register it as a resource.
    let engine = RulesEngine::new(Player::new("Player 1", 20), Player::new("Player 2", 20));
    info!(
        "Rules engine created: {} ({} life) vs {} ({} life)",
        engine.players[0].name,
        engine.players[0].life,
        engine.players[1].name,
        engine.players[1].life,
    );
    commands.insert_resource(RulesEngineResource(engine));
}

fn log_game_state(state: Res<GameState>, mut ran: Local<bool>) {
    if !*ran {
        info!("Game started. Turn: {}", state.turn);
        *ran = true;
    }
}

/// Runs one demo round of the rules engine to exercise the full turn flow.
///
/// Simulates one complete round: draw → play → battle → end.  This system runs
/// once to demonstrate the rules engine integrating with Bevy's ECS scheduler.
fn run_demo_round(engine_res: Option<ResMut<RulesEngineResource>>, mut ran: Local<bool>) {
    if *ran {
        return;
    }
    let Some(mut res) = engine_res else {
        return;
    };
    let engine = &mut res.0;

    // Demo cards for this round: attacker plays high, defender reveals low.
    let attacker_card = Card::new("Ace", 14);
    let defender_card = Card::new("Two", 2);

    let active_name = engine.players[engine.active_player].name.clone();
    info!(
        "Demo round {} — active player: {}",
        engine.round, active_name
    );

    // Phase 1: Draw
    if let Err(e) = engine.draw_card(attacker_card.clone()) {
        warn!("draw_card failed: {e}");
        *ran = true;
        return;
    }

    // Phase 2: Play
    let played = match engine.play_card(0) {
        Ok(card) => card,
        Err(e) => {
            warn!("play_card failed: {e}");
            *ran = true;
            return;
        }
    };
    info!(
        "{active_name} plays {} (value: {})",
        played.name, played.value
    );

    // Phase 3: Battle
    match engine.resolve_battle(&played, &defender_card) {
        Ok(outcome) => info!("Battle outcome: {:?}", outcome),
        Err(e) => {
            warn!("resolve_battle failed: {e}");
            *ran = true;
            return;
        }
    }

    // Phase 4: End round
    if let Err(e) = engine.end_turn() {
        warn!("end_turn failed: {e}");
        *ran = true;
        return;
    }

    // Check game state after the round.
    if engine.is_game_over() {
        if let Some(winner_idx) = engine.winner() {
            info!("Game over! Winner: {}", engine.players[winner_idx].name);
        }
    } else {
        info!(
            "Round complete — now at round {}. Player 1 life={}, Player 2 life={}",
            engine.round, engine.players[0].life, engine.players[1].life,
        );
    }

    *ran = true;
}

/// Spawns the game UI when entering the InGame state.
fn spawn_game_ui(mut commands: Commands) {
    commands
        .spawn((
            GameUiRoot,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(20.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                ..default()
            },
        ))
        .with_children(|root| {
            // Title
            root.spawn(TextBundle::from_section(
                "Controlled Chaos - Game Active",
                TextStyle {
                    font_size: 32.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            // Player 1 Info
            root.spawn((
                Player1InfoText,
                TextBundle::from_section(
                    "Player 1: 20 Life",
                    TextStyle {
                        font_size: 24.0,
                        color: Color::srgb(0.3, 0.8, 0.3),
                        ..default()
                    },
                ),
            ));

            // Player 2 Info
            root.spawn((
                Player2InfoText,
                TextBundle::from_section(
                    "Player 2: 20 Life",
                    TextStyle {
                        font_size: 24.0,
                        color: Color::srgb(0.8, 0.3, 0.3),
                        ..default()
                    },
                ),
            ));

            // Game Phase Info
            root.spawn((
                GamePhaseText,
                TextBundle::from_section(
                    "Phase: Draw | Round: 1",
                    TextStyle {
                        font_size: 20.0,
                        color: Color::srgb(0.8, 0.8, 0.3),
                        ..default()
                    },
                ),
            ));

            // Instructions
            root.spawn(TextBundle::from_section(
                "Demo game running in background. Check console for details.",
                TextStyle {
                    font_size: 16.0,
                    color: Color::srgb(0.6, 0.6, 0.6),
                    ..default()
                },
            ));
        });
}

/// Updates the game UI to reflect the current state of the rules engine.
#[allow(clippy::type_complexity)]
fn update_game_ui(
    engine_res: Option<Res<RulesEngineResource>>,
    mut p1_query: Query<
        &mut Text,
        (
            With<Player1InfoText>,
            Without<Player2InfoText>,
            Without<GamePhaseText>,
        ),
    >,
    mut p2_query: Query<
        &mut Text,
        (
            With<Player2InfoText>,
            Without<Player1InfoText>,
            Without<GamePhaseText>,
        ),
    >,
    mut phase_query: Query<
        &mut Text,
        (
            With<GamePhaseText>,
            Without<Player1InfoText>,
            Without<Player2InfoText>,
        ),
    >,
) {
    let Some(res) = engine_res else { return };
    let engine = &res.0;

    // Update Player 1 info
    if let Ok(mut text) = p1_query.get_single_mut() {
        text.sections[0].value = format!(
            "{}: {} Life",
            engine.players[0].name, engine.players[0].life
        );
    }

    // Update Player 2 info
    if let Ok(mut text) = p2_query.get_single_mut() {
        text.sections[0].value = format!(
            "{}: {} Life",
            engine.players[1].name, engine.players[1].life
        );
    }

    // Update phase info
    if let Ok(mut text) = phase_query.get_single_mut() {
        let phase_name = match engine.phase {
            crate::rules::TurnPhase::Draw => "Draw",
            crate::rules::TurnPhase::Play => "Play",
            crate::rules::TurnPhase::Battle => "Battle",
            crate::rules::TurnPhase::End => "End",
        };
        text.sections[0].value = format!(
            "Phase: {} | Round: {} | Active: {}",
            phase_name, engine.round, engine.players[engine.active_player].name
        );
    }
}

/// Despawns the game UI when exiting the InGame state.
fn despawn_game_ui(mut commands: Commands, query: Query<Entity, With<GameUiRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
