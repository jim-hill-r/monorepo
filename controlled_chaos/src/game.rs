use bevy::prelude::*;

use crate::card::{Card, CardCategory};
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

/// Marks the text displaying Player 1's civilian count.
#[derive(Component, Debug)]
pub struct Player1CiviliansText;

/// Marks the text displaying Player 2's civilian count.
#[derive(Component, Debug)]
pub struct Player2CiviliansText;

/// Marks the text displaying Player 1's happiness score.
#[derive(Component, Debug)]
pub struct Player1HappinessText;

/// Marks the text displaying Player 2's happiness score.
#[derive(Component, Debug)]
pub struct Player2HappinessText;

/// Marks the text displaying the current game phase.
#[derive(Component, Debug)]
pub struct GamePhaseText;

fn setup(mut commands: Commands) {
    // Build a card library with the available cards, using proper categories
    // from the rulebook.
    let mut library = CardLibrary::new();
    library.register(Card::with_category(
        "Ace of Spades",
        14,
        CardCategory::Technology,
    ));
    library.register(Card::with_category("NASA", 13, CardCategory::Government));
    library.register(Card::with_category(
        "Biodome",
        12,
        CardCategory::Environment,
    ));
    library.register(Card::with_category(
        "Stock Market",
        11,
        CardCategory::Economy,
    ));
    library.register(Card::with_category("Asteroid", 10, CardCategory::Crisis));
    library.register(Card::with_category("Engineer", 9, CardCategory::Profession));
    library.register(Card::with_category("Suburb", 8, CardCategory::Civilian));
    library.register(Card::with_category(
        "Space Station",
        7,
        CardCategory::Society,
    ));

    info!(
        "Card library contains {} cards: {}",
        library.len(),
        library.card_names().join(", ")
    );

    // Use DeckBuilder to compose a player deck (max 2 copies of any card).
    let mut builder = DeckBuilder::new().with_max_copies(2);
    for name in [
        "Ace of Spades",
        "NASA",
        "Biodome",
        "Stock Market",
        "Ace of Spades",
    ] {
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
        info!(
            "Drew card: {} (value: {}, category: {})",
            card.name,
            card.value,
            card.category.label()
        );
    }

    // Initialise the rules engine with two players and register it as a resource.
    let engine = RulesEngine::new(Player::new("Player 1", 20), Player::new("Player 2", 20));
    info!(
        "Rules engine created: {} ({} life, {} civilians, {} happiness) vs {} ({} life, {} civilians, {} happiness)",
        engine.players[0].name,
        engine.players[0].life,
        engine.players[0].civilians,
        engine.players[0].happiness,
        engine.players[1].name,
        engine.players[1].life,
        engine.players[1].civilians,
        engine.players[1].happiness,
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

    // Demo cards for this round: attacker plays a high Technology card, defender
    // reveals a low-value card.
    let attacker_card = Card::with_category("Ace of Spades", 14, CardCategory::Technology);
    let defender_card = Card::new("Two of Clubs", 2);

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
        "{active_name} plays {} ({}, value: {})",
        played.name,
        played.category.label(),
        played.value
    );

    // Phase 3: Battle
    match engine.resolve_battle(&played, &defender_card) {
        Ok(outcome) => {
            info!("Battle outcome: {:?}", outcome);
            // Winning the battle gives the active player a small happiness boost.
            match outcome {
                crate::rules::BattleOutcome::AttackerWins => {
                    engine.players[engine.active_player].adjust_happiness(5);
                }
                crate::rules::BattleOutcome::DefenderWins => {
                    let defender_idx = engine.inactive_player();
                    engine.players[defender_idx].adjust_happiness(5);
                }
                crate::rules::BattleOutcome::Draw => {}
            }
        }
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

    // Simulate deck exhaustion: the demo uses a single round, so signal that
    // the action deck is now empty.  In a full game this would be called by
    // the deck management system when the last card is drawn.
    engine.signal_deck_exhausted();

    // Check game state after the round.
    if engine.is_game_over() {
        if let Some(winner_idx) = engine.winner() {
            info!("Game over! Winner: {}", engine.players[winner_idx].name);
        } else {
            info!("Game over! It's a draw.");
        }
    } else {
        info!(
            "Round complete — now at round {}. Player 1 life={}, civilians={}, happiness={} | Player 2 life={}, civilians={}, happiness={}",
            engine.round,
            engine.players[0].life,
            engine.players[0].civilians,
            engine.players[0].happiness,
            engine.players[1].life,
            engine.players[1].civilians,
            engine.players[1].happiness,
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

            // Player 1 Info (life)
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

            // Player 1 civilians
            root.spawn((
                Player1CiviliansText,
                TextBundle::from_section(
                    "  Civilians: 2",
                    TextStyle {
                        font_size: 18.0,
                        color: Color::srgb(0.3, 0.7, 0.7),
                        ..default()
                    },
                ),
            ));

            // Player 1 happiness
            root.spawn((
                Player1HappinessText,
                TextBundle::from_section(
                    "  Happiness: 50",
                    TextStyle {
                        font_size: 18.0,
                        color: Color::srgb(0.8, 0.8, 0.3),
                        ..default()
                    },
                ),
            ));

            // Player 2 Info (life)
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

            // Player 2 civilians
            root.spawn((
                Player2CiviliansText,
                TextBundle::from_section(
                    "  Civilians: 2",
                    TextStyle {
                        font_size: 18.0,
                        color: Color::srgb(0.3, 0.5, 0.7),
                        ..default()
                    },
                ),
            ));

            // Player 2 happiness
            root.spawn((
                Player2HappinessText,
                TextBundle::from_section(
                    "  Happiness: 50",
                    TextStyle {
                        font_size: 18.0,
                        color: Color::srgb(0.8, 0.6, 0.3),
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

            // Win condition reminder
            root.spawn(TextBundle::from_section(
                "Win: Highest happiness when deck runs out. Lose: No civilians left.",
                TextStyle {
                    font_size: 14.0,
                    color: Color::srgb(0.6, 0.6, 0.6),
                    ..default()
                },
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
#[allow(clippy::too_many_arguments)]
fn update_game_ui(
    engine_res: Option<Res<RulesEngineResource>>,
    mut p1_query: Query<
        &mut Text,
        (
            With<Player1InfoText>,
            Without<Player2InfoText>,
            Without<GamePhaseText>,
            Without<Player1CiviliansText>,
            Without<Player2CiviliansText>,
            Without<Player1HappinessText>,
            Without<Player2HappinessText>,
        ),
    >,
    mut p2_query: Query<
        &mut Text,
        (
            With<Player2InfoText>,
            Without<Player1InfoText>,
            Without<GamePhaseText>,
            Without<Player1CiviliansText>,
            Without<Player2CiviliansText>,
            Without<Player1HappinessText>,
            Without<Player2HappinessText>,
        ),
    >,
    mut p1_civ_query: Query<
        &mut Text,
        (
            With<Player1CiviliansText>,
            Without<Player1InfoText>,
            Without<Player2InfoText>,
            Without<GamePhaseText>,
            Without<Player2CiviliansText>,
            Without<Player1HappinessText>,
            Without<Player2HappinessText>,
        ),
    >,
    mut p2_civ_query: Query<
        &mut Text,
        (
            With<Player2CiviliansText>,
            Without<Player1InfoText>,
            Without<Player2InfoText>,
            Without<GamePhaseText>,
            Without<Player1CiviliansText>,
            Without<Player1HappinessText>,
            Without<Player2HappinessText>,
        ),
    >,
    mut p1_happiness_query: Query<
        &mut Text,
        (
            With<Player1HappinessText>,
            Without<Player1InfoText>,
            Without<Player2InfoText>,
            Without<GamePhaseText>,
            Without<Player1CiviliansText>,
            Without<Player2CiviliansText>,
            Without<Player2HappinessText>,
        ),
    >,
    mut p2_happiness_query: Query<
        &mut Text,
        (
            With<Player2HappinessText>,
            Without<Player1InfoText>,
            Without<Player2InfoText>,
            Without<GamePhaseText>,
            Without<Player1CiviliansText>,
            Without<Player2CiviliansText>,
            Without<Player1HappinessText>,
        ),
    >,
    mut phase_query: Query<
        &mut Text,
        (
            With<GamePhaseText>,
            Without<Player1InfoText>,
            Without<Player2InfoText>,
            Without<Player1CiviliansText>,
            Without<Player2CiviliansText>,
            Without<Player1HappinessText>,
            Without<Player2HappinessText>,
        ),
    >,
) {
    let Some(res) = engine_res else { return };
    let engine = &res.0;

    // Update Player 1 life
    if let Ok(mut text) = p1_query.get_single_mut() {
        text.sections[0].value = format!(
            "{}: {} Life",
            engine.players[0].name, engine.players[0].life
        );
    }

    // Update Player 2 life
    if let Ok(mut text) = p2_query.get_single_mut() {
        text.sections[0].value = format!(
            "{}: {} Life",
            engine.players[1].name, engine.players[1].life
        );
    }

    // Update Player 1 civilians
    if let Ok(mut text) = p1_civ_query.get_single_mut() {
        text.sections[0].value = format!("  Civilians: {}", engine.players[0].civilians);
    }

    // Update Player 2 civilians
    if let Ok(mut text) = p2_civ_query.get_single_mut() {
        text.sections[0].value = format!("  Civilians: {}", engine.players[1].civilians);
    }

    // Update Player 1 happiness
    if let Ok(mut text) = p1_happiness_query.get_single_mut() {
        text.sections[0].value = format!("  Happiness: {}", engine.players[0].happiness);
    }

    // Update Player 2 happiness
    if let Ok(mut text) = p2_happiness_query.get_single_mut() {
        text.sections[0].value = format!("  Happiness: {}", engine.players[1].happiness);
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
