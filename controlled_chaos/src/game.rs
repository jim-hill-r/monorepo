use bevy::prelude::*;

use crate::ai::{AiOpponent, choose_attack_card, choose_defense_card};
use crate::card::{Card, CardCategory};
use crate::card_library::CardLibrary;
use crate::deck::{Deck, DeckBuilder};
use crate::rules::{BattleOutcome, Player, RulesEngine, TurnPhase};
use crate::state::AppState;

/// Plugin that sets up the core game systems.
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameState::default())
            .add_systems(OnEnter(AppState::InGame), (setup, spawn_game_ui))
            .add_systems(
                Update,
                (
                    log_game_state,
                    // Phase handlers first: they mutate RulesEngineResource and insert/
                    // remove AttackerCardResource via Commands.  apply_deferred flushes
                    // those commands so the UI systems below read the latest state.
                    (
                        handle_draw_phase,
                        handle_play_phase,
                        handle_battle_phase,
                        handle_end_phase,
                        // AI handlers run after human handlers so human input is
                        // processed first.  The AI acts automatically when it is
                        // Player 2's turn or when it needs to defend.
                        ai_handle_draw_phase,
                        ai_handle_play_phase,
                        ai_handle_battle_phase,
                        ai_handle_end_phase,
                    )
                        .chain(),
                    apply_deferred,
                    // UI systems run after deferred commands are applied.
                    (update_game_ui, update_pvp_ui).chain(),
                )
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                OnExit(AppState::InGame),
                (despawn_game_ui, cleanup_pvp_resources),
            );
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

/// The shared action deck both players draw from.
#[derive(Resource)]
pub struct ActionDeckResource(pub Deck);

/// The card played by the active player during the Play phase.
///
/// This resource is inserted when the active player commits their card and
/// removed after the battle is resolved.
#[derive(Resource)]
pub struct AttackerCardResource(pub Card);

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

/// Marks the text displaying the active player's hand.
#[derive(Component, Debug)]
pub struct HandDisplayText;

/// Marks the text displaying contextual instructions for the active player.
#[derive(Component, Debug)]
pub struct InstructionsText;

/// Number of starting cards dealt to each player at game setup.
pub const STARTING_HAND_SIZE: usize = 3;

/// Placeholder card used when a player has no cards left in hand during the
/// battle phase.  A value of `0` ensures it always loses.
const EMPTY_HAND_CARD: &str = "Empty Hand";

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
    library.register(Card::with_category(
        "Solar Panel",
        6,
        CardCategory::Technology,
    ));
    library.register(Card::with_category(
        "City Hall",
        5,
        CardCategory::Government,
    ));
    library.register(Card::with_category("Forest", 4, CardCategory::Environment));
    library.register(Card::with_category("Trade Route", 3, CardCategory::Economy));
    library.register(Card::with_category("Flood", 2, CardCategory::Crisis));
    library.register(Card::with_category("Farmer", 1, CardCategory::Profession));

    info!(
        "Card library contains {} cards: {}",
        library.len(),
        library.card_names().join(", ")
    );

    // Build the shared action deck (max 2 copies of any card).
    let mut builder = DeckBuilder::new().with_max_copies(2);
    for name in [
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
    ] {
        if let Some(card) = library.get(name) {
            builder.add_card(card);
        }
    }
    let mut action_deck: Deck = builder.build();
    action_deck.shuffle();

    info!(
        "Action deck initialized with {} cards.",
        action_deck.remaining()
    );

    // Initialise the rules engine with two players.
    let mut engine = RulesEngine::new(Player::new("Player 1", 20), Player::new("CPU", 20));

    // Deal starting hands to each player.
    for _ in 0..STARTING_HAND_SIZE {
        if let Some(card) = action_deck.draw() {
            engine.players[0].receive_card(card);
        }
        if let Some(card) = action_deck.draw() {
            engine.players[1].receive_card(card);
        }
    }

    info!(
        "Players ready — {} has {} cards, {} has {} cards. {} cards remain in deck.",
        engine.players[0].name,
        engine.players[0].hand.len(),
        engine.players[1].name,
        engine.players[1].hand.len(),
        action_deck.remaining(),
    );

    commands.insert_resource(RulesEngineResource(engine));
    commands.insert_resource(ActionDeckResource(action_deck));
    commands.insert_resource(AiOpponent);
}

fn log_game_state(state: Res<GameState>, mut ran: Local<bool>) {
    if !*ran {
        info!("Game started. Turn: {}", state.turn);
        *ran = true;
    }
}

/// Handles player input during the Draw phase.
///
/// The active player presses **Space** to draw the top card from the shared
/// action deck.  If the deck is empty the game signals deck exhaustion and the
/// game ends.
///
/// In single-player mode (when [`AiOpponent`] is present) this handler only
/// responds when it is Player 1's turn (`active_player == 0`).
fn handle_draw_phase(
    keys: Res<ButtonInput<KeyCode>>,
    engine_res: Option<ResMut<RulesEngineResource>>,
    deck_res: Option<ResMut<ActionDeckResource>>,
    ai: Option<Res<AiOpponent>>,
) {
    let (Some(mut engine_res), Some(mut deck_res)) = (engine_res, deck_res) else {
        return;
    };
    let engine = &mut engine_res.0;

    if engine.phase != TurnPhase::Draw || engine.is_game_over() {
        return;
    }

    // In single-player mode, only Player 1 controls this phase.
    if ai.is_some() && engine.active_player != 0 {
        return;
    }

    if keys.just_pressed(KeyCode::Space) {
        let deck = &mut deck_res.0;
        if let Some(card) = deck.draw() {
            if let Err(e) = engine.draw_card(card) {
                warn!("draw_card failed: {e}");
            }
        } else {
            // Deck is empty — end the game by scoring on happiness.
            engine.signal_deck_exhausted();
        }
    }
}

/// Handles player input during the Play phase.
///
/// The active player presses **1–8** to select a card from their hand to play
/// as the attacker.  The selected card is stored in [`AttackerCardResource`]
/// and removed from the player's hand.
///
/// In single-player mode (when [`AiOpponent`] is present) this handler only
/// responds when it is Player 1's turn (`active_player == 0`).
fn handle_play_phase(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    engine_res: Option<ResMut<RulesEngineResource>>,
    ai: Option<Res<AiOpponent>>,
) {
    let Some(mut engine_res) = engine_res else {
        return;
    };
    let engine = &mut engine_res.0;

    if engine.phase != TurnPhase::Play || engine.is_game_over() {
        return;
    }

    // In single-player mode, only Player 1 controls this phase.
    if ai.is_some() && engine.active_player != 0 {
        return;
    }

    let hand_len = engine.players[engine.active_player].hand.len();
    if hand_len == 0 {
        return;
    }

    let key_indices = [
        (KeyCode::Digit1, 0usize),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3),
        (KeyCode::Digit5, 4),
        (KeyCode::Digit6, 5),
        (KeyCode::Digit7, 6),
        (KeyCode::Digit8, 7),
    ];

    for (key, idx) in &key_indices {
        if keys.just_pressed(*key) && *idx < hand_len {
            match engine.play_card(*idx) {
                Ok(card) => {
                    info!(
                        "{} plays {} ({}, value: {})",
                        engine.players[engine.active_player].name,
                        card.name,
                        card.category.label(),
                        card.value
                    );
                    commands.insert_resource(AttackerCardResource(card));
                }
                Err(e) => warn!("play_card failed: {e}"),
            }
            return;
        }
    }
}

/// Handles player input during the Battle phase.
///
/// The *defending* player (the one who is not active) presses **1–8** to
/// select a card from their hand to defend with.  The two cards are then
/// compared by [`RulesEngine::resolve_battle`], damage is applied, and the
/// [`AttackerCardResource`] is removed to signal that the battle is complete.
///
/// If the defender has no cards in hand they automatically lose the battle.
///
/// In single-player mode (when [`AiOpponent`] is present) this handler only
/// responds when Player 1 is the defender (`inactive_player == 0`), i.e. when
/// the AI is attacking.  When the AI is defending, [`ai_handle_battle_phase`]
/// handles the response instead.
fn handle_battle_phase(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    engine_res: Option<ResMut<RulesEngineResource>>,
    attacker_res: Option<Res<AttackerCardResource>>,
    ai: Option<Res<AiOpponent>>,
) {
    let (Some(mut engine_res), Some(attacker_res)) = (engine_res, attacker_res) else {
        return;
    };
    let engine = &mut engine_res.0;

    if engine.phase != TurnPhase::Battle || engine.is_game_over() {
        return;
    }

    let defender_idx = engine.inactive_player();

    // In single-player mode, only handle input when Player 1 is defending.
    if ai.is_some() && defender_idx != 0 {
        return;
    }

    let defender_hand_len = engine.players[defender_idx].hand.len();

    // If the defender has no cards, the attacker wins automatically.
    if defender_hand_len == 0 {
        let attacker_card = attacker_res.0.clone();
        let empty_hand_card = Card::new(EMPTY_HAND_CARD, 0);
        apply_battle_result(engine, &attacker_card, &empty_hand_card);
        commands.remove_resource::<AttackerCardResource>();
        return;
    }

    let key_indices = [
        (KeyCode::Digit1, 0usize),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3),
        (KeyCode::Digit5, 4),
        (KeyCode::Digit6, 5),
        (KeyCode::Digit7, 6),
        (KeyCode::Digit8, 7),
    ];

    for (key, idx) in &key_indices {
        if keys.just_pressed(*key) && *idx < defender_hand_len {
            let Some(defender_card) = engine.players[defender_idx].play_card(*idx) else {
                return;
            };
            let attacker_card = attacker_res.0.clone();
            apply_battle_result(engine, &attacker_card, &defender_card);
            commands.remove_resource::<AttackerCardResource>();
            return;
        }
    }
}

/// Resolves the battle between `attacker` and `defender` and adjusts happiness.
///
/// Called by [`handle_battle_phase`] after both cards have been selected.  The
/// winner gains +5 happiness.
fn apply_battle_result(engine: &mut RulesEngine, attacker: &Card, defender: &Card) {
    match engine.resolve_battle(attacker, defender) {
        Ok(outcome) => {
            info!("Battle outcome: {outcome:?}");
            match outcome {
                BattleOutcome::AttackerWins => {
                    engine.players[engine.active_player].adjust_happiness(5);
                }
                BattleOutcome::DefenderWins => {
                    let defender_idx = engine.inactive_player();
                    engine.players[defender_idx].adjust_happiness(5);
                }
                BattleOutcome::Draw => {}
            }
        }
        Err(e) => warn!("resolve_battle failed: {e}"),
    }
}

/// Handles player input during the End phase.
///
/// Either player presses **Space** to end the turn.  If the game is over no
/// input is accepted and the result is displayed on screen.
///
/// In single-player mode (when [`AiOpponent`] is present) this handler only
/// responds when it is Player 1's turn.  When it is the AI's turn,
/// [`ai_handle_end_phase`] ends the turn automatically.
fn handle_end_phase(
    keys: Res<ButtonInput<KeyCode>>,
    engine_res: Option<ResMut<RulesEngineResource>>,
    ai: Option<Res<AiOpponent>>,
) {
    let Some(mut engine_res) = engine_res else {
        return;
    };
    let engine = &mut engine_res.0;

    if engine.phase != TurnPhase::End || engine.is_game_over() {
        return;
    }

    // In single-player mode, only Player 1 ends the turn manually.
    if ai.is_some() && engine.active_player != 0 {
        return;
    }

    if keys.just_pressed(KeyCode::Space)
        && let Err(e) = engine.end_turn()
    {
        warn!("end_turn failed: {e}");
    }
}

/// Removes PvP-specific resources when leaving the InGame state.
fn cleanup_pvp_resources(mut commands: Commands) {
    commands.remove_resource::<ActionDeckResource>();
    commands.remove_resource::<AttackerCardResource>();
    commands.remove_resource::<AiOpponent>();
}

// ── AI systems ────────────────────────────────────────────────────────────────

/// Handles AI actions during the Draw phase.
///
/// When the AI is the active player (index 1), it automatically draws a card
/// from the action deck instead of waiting for keyboard input.
fn ai_handle_draw_phase(
    engine_res: Option<ResMut<RulesEngineResource>>,
    deck_res: Option<ResMut<ActionDeckResource>>,
    ai: Option<Res<AiOpponent>>,
) {
    let (Some(_ai), Some(mut engine_res), Some(mut deck_res)) = (ai, engine_res, deck_res) else {
        return;
    };
    let engine = &mut engine_res.0;

    if engine.phase != TurnPhase::Draw || engine.is_game_over() || engine.active_player != 1 {
        return;
    }

    let deck = &mut deck_res.0;
    if let Some(card) = deck.draw() {
        if let Err(e) = engine.draw_card(card) {
            warn!("AI draw_card failed: {e}");
        } else {
            info!("CPU draws a card.");
        }
    } else {
        engine.signal_deck_exhausted();
    }
}

/// Handles AI actions during the Play phase.
///
/// When the AI is the active player (index 1), it automatically plays the
/// highest-value card from its hand using [`choose_attack_card`].
fn ai_handle_play_phase(
    mut commands: Commands,
    engine_res: Option<ResMut<RulesEngineResource>>,
    ai: Option<Res<AiOpponent>>,
) {
    let (Some(_ai), Some(mut engine_res)) = (ai, engine_res) else {
        return;
    };
    let engine = &mut engine_res.0;

    if engine.phase != TurnPhase::Play || engine.is_game_over() || engine.active_player != 1 {
        return;
    }

    let hand_len = engine.players[1].hand.len();
    if hand_len == 0 {
        return;
    }

    let idx = choose_attack_card(&engine.players[1].hand);
    match engine.play_card(idx) {
        Ok(card) => {
            info!("CPU plays {} (value: {})", card.name, card.value);
            commands.insert_resource(AttackerCardResource(card));
        }
        Err(e) => warn!("AI play_card failed: {e}"),
    }
}

/// Handles AI defense during the Battle phase.
///
/// When Player 1 is attacking (`active_player == 0`) and the AI is the
/// defender (index 1), the AI automatically chooses a card to defend with
/// using [`choose_defense_card`].  If the AI has no cards in hand the attacker
/// wins automatically.
fn ai_handle_battle_phase(
    mut commands: Commands,
    engine_res: Option<ResMut<RulesEngineResource>>,
    attacker_res: Option<Res<AttackerCardResource>>,
    ai: Option<Res<AiOpponent>>,
) {
    let (Some(_ai), Some(mut engine_res), Some(attacker_res)) = (ai, engine_res, attacker_res)
    else {
        return;
    };
    let engine = &mut engine_res.0;

    if engine.phase != TurnPhase::Battle || engine.is_game_over() {
        return;
    }

    // Only act when the AI (index 1) is defending.
    if engine.inactive_player() != 1 {
        return;
    }

    let attacker_card = attacker_res.0.clone();
    let defender_hand_len = engine.players[1].hand.len();

    if defender_hand_len == 0 {
        apply_battle_result(engine, &attacker_card, &Card::new(EMPTY_HAND_CARD, 0));
        commands.remove_resource::<AttackerCardResource>();
        return;
    }

    let idx = choose_defense_card(&engine.players[1].hand, attacker_card.value);
    let Some(defender_card) = engine.players[1].play_card(idx) else {
        return;
    };
    info!(
        "CPU defends with {} (value: {})",
        defender_card.name, defender_card.value
    );
    apply_battle_result(engine, &attacker_card, &defender_card);
    commands.remove_resource::<AttackerCardResource>();
}

/// Handles AI actions during the End phase.
///
/// When the AI is the active player (index 1), it automatically ends its turn.
fn ai_handle_end_phase(
    engine_res: Option<ResMut<RulesEngineResource>>,
    ai: Option<Res<AiOpponent>>,
) {
    let (Some(_ai), Some(mut engine_res)) = (ai, engine_res) else {
        return;
    };
    let engine = &mut engine_res.0;

    if engine.phase != TurnPhase::End || engine.is_game_over() || engine.active_player != 1 {
        return;
    }

    if let Err(e) = engine.end_turn() {
        warn!("AI end_turn failed: {e}");
    }
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
                    row_gap: Val::Px(12.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                ..default()
            },
        ))
        .with_children(|root| {
            // Title
            root.spawn(TextBundle::from_section(
                "Controlled Chaos — Player vs CPU",
                TextStyle {
                    font_size: 28.0,
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
                        font_size: 22.0,
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
                        font_size: 16.0,
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
                        font_size: 16.0,
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
                        font_size: 22.0,
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
                        font_size: 16.0,
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
                        font_size: 16.0,
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
                        font_size: 18.0,
                        color: Color::srgb(0.8, 0.8, 0.3),
                        ..default()
                    },
                ),
            ));

            // Current player's hand
            root.spawn((
                HandDisplayText,
                TextBundle::from_section(
                    "Hand: (loading…)",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::srgb(0.9, 0.9, 0.9),
                        ..default()
                    },
                ),
            ));

            // Dynamic instructions
            root.spawn((
                InstructionsText,
                TextBundle::from_section(
                    "Press SPACE to draw a card",
                    TextStyle {
                        font_size: 18.0,
                        color: Color::srgb(0.4, 0.9, 0.4),
                        ..default()
                    },
                ),
            ));

            // Win condition reminder
            root.spawn(TextBundle::from_section(
                "Win: Highest happiness when deck runs out.  Lose: Life or civilians reach 0.",
                TextStyle {
                    font_size: 12.0,
                    color: Color::srgb(0.5, 0.5, 0.5),
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
            Without<HandDisplayText>,
            Without<InstructionsText>,
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
            Without<HandDisplayText>,
            Without<InstructionsText>,
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
            Without<HandDisplayText>,
            Without<InstructionsText>,
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
            Without<HandDisplayText>,
            Without<InstructionsText>,
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
            Without<HandDisplayText>,
            Without<InstructionsText>,
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
            Without<HandDisplayText>,
            Without<InstructionsText>,
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
            Without<HandDisplayText>,
            Without<InstructionsText>,
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
            TurnPhase::Draw => "Draw",
            TurnPhase::Play => "Play",
            TurnPhase::Battle => "Battle",
            TurnPhase::End => "End",
        };
        text.sections[0].value = format!(
            "Phase: {} | Round: {} | Active: {}",
            phase_name, engine.round, engine.players[engine.active_player].name
        );
    }
}

/// Updates the hand display and instructions text for the current turn.
///
/// In single-player mode the AI's hand is hidden and AI turns display a
/// "CPU is acting…" status message instead of player instructions.
fn update_pvp_ui(
    engine_res: Option<Res<RulesEngineResource>>,
    deck_res: Option<Res<ActionDeckResource>>,
    attacker_res: Option<Res<AttackerCardResource>>,
    ai: Option<Res<AiOpponent>>,
    mut hand_query: Query<&mut Text, (With<HandDisplayText>, Without<InstructionsText>)>,
    mut instr_query: Query<&mut Text, (With<InstructionsText>, Without<HandDisplayText>)>,
) {
    let Some(res) = engine_res else {
        return;
    };
    let engine = &res.0;

    let is_ai_present = ai.is_some();
    let deck_remaining = deck_res.as_ref().map(|d| d.0.remaining()).unwrap_or(0);

    // Determine which player's hand to display.
    let (hand_player_idx, hand_label) = match engine.phase {
        TurnPhase::Battle => {
            // Show the defender's hand so they can pick a card to defend with.
            let def_idx = engine.inactive_player();
            (
                def_idx,
                format!("{}'s hand (defending):", engine.players[def_idx].name),
            )
        }
        _ => {
            // All other phases show the active player's hand.
            let idx = engine.active_player;
            (idx, format!("{}'s hand:", engine.players[idx].name))
        }
    };

    // In single-player mode the AI's hand is hidden.
    let hand_text = if is_ai_present && hand_player_idx == 1 {
        format!("{hand_label}\n  (hidden)")
    } else {
        let hand = &engine.players[hand_player_idx].hand;
        if hand.is_empty() {
            format!("{hand_label}\n  (empty)")
        } else {
            let cards: Vec<String> = hand
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    format!(
                        "  [{}] {} — {} ({})",
                        i + 1,
                        c.name,
                        c.category.label(),
                        c.value
                    )
                })
                .collect();
            format!("{hand_label}\n{}", cards.join("\n"))
        }
    };

    if let Ok(mut text) = hand_query.get_single_mut() {
        text.sections[0].value = hand_text;
    }

    // Determine whether the AI is currently acting.
    let ai_is_acting = is_ai_present
        && !engine.is_game_over()
        && match engine.phase {
            TurnPhase::Battle => engine.inactive_player() == 1,
            _ => engine.active_player == 1,
        };

    // Build the instruction string based on current phase.
    let instructions = if engine.is_game_over() {
        match engine.winner() {
            Some(idx) => {
                let loser_idx = 1 - idx;
                if engine.deck_exhausted {
                    // Deck ran out — winner has higher happiness.
                    format!(
                        "GAME OVER — {} wins! (Happiness: {} vs {})",
                        engine.players[idx].name,
                        engine.players[idx].happiness,
                        engine.players[loser_idx].happiness,
                    )
                } else {
                    // Immediate defeat — loser lost all life or civilians.
                    let reason = if engine.players[loser_idx].life == 0 {
                        "ran out of life"
                    } else {
                        "lost all civilians"
                    };
                    format!(
                        "GAME OVER — {} wins! ({} {})",
                        engine.players[idx].name, engine.players[loser_idx].name, reason,
                    )
                }
            }
            None => "GAME OVER — It's a draw!".to_string(),
        }
    } else if ai_is_acting {
        "CPU is taking its turn…".to_string()
    } else {
        match engine.phase {
            TurnPhase::Draw => format!(
                "{}: Press SPACE to draw  ({} cards in deck)",
                engine.players[engine.active_player].name, deck_remaining
            ),
            TurnPhase::Play => {
                let hand_len = engine.players[engine.active_player].hand.len();
                format!(
                    "{}: Press 1–{} to play a card",
                    engine.players[engine.active_player].name, hand_len
                )
            }
            TurnPhase::Battle => {
                let def_idx = engine.inactive_player();
                let def_hand_len = engine.players[def_idx].hand.len();
                let attacker_info = attacker_res
                    .as_ref()
                    .map(|r| format!("{} ({})", r.0.name, r.0.value))
                    .unwrap_or_default();
                if def_hand_len == 0 {
                    format!(
                        "{} attacks with {}. {} has no cards — attacker wins!",
                        engine.players[engine.active_player].name,
                        attacker_info,
                        engine.players[def_idx].name
                    )
                } else {
                    format!(
                        "{} attacks with {}.  {}: Press 1–{} to defend",
                        engine.players[engine.active_player].name,
                        attacker_info,
                        engine.players[def_idx].name,
                        def_hand_len
                    )
                }
            }
            TurnPhase::End => format!(
                "Turn over.  Press SPACE to pass to {}",
                engine.players[engine.inactive_player()].name
            ),
        }
    };

    if let Ok(mut text) = instr_query.get_single_mut() {
        text.sections[0].value = instructions;
    }
}

/// Despawns the game UI when exiting the InGame state.
fn despawn_game_ui(mut commands: Commands, query: Query<Entity, With<GameUiRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    fn make_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    fn make_engine_with_hands() -> RulesEngine {
        let mut engine = RulesEngine::new(Player::new("Alice", 20), Player::new("Bob", 20));
        engine.players[0].receive_card(Card::new("Ace", 14));
        engine.players[0].receive_card(Card::new("King", 13));
        engine.players[1].receive_card(Card::new("Queen", 12));
        engine.players[1].receive_card(Card::new("Jack", 11));
        engine
    }

    // ── apply_battle_result ───────────────────────────────────────────────────

    #[test]
    fn apply_battle_result_attacker_wins_gives_attacker_happiness() {
        let mut engine = make_engine_with_hands();
        // Advance to Battle phase.
        engine.draw_card(Card::new("Ace", 14)).unwrap();
        engine.play_card(0).unwrap();

        let attacker = Card::new("Ace", 14);
        let defender = Card::new("Two", 2);
        let initial_happiness = engine.players[0].happiness;
        apply_battle_result(&mut engine, &attacker, &defender);
        assert_eq!(engine.players[0].happiness, initial_happiness + 5);
    }

    #[test]
    fn apply_battle_result_defender_wins_gives_defender_happiness() {
        let mut engine = make_engine_with_hands();
        engine.draw_card(Card::new("Two", 2)).unwrap();
        engine.play_card(0).unwrap();

        let attacker = Card::new("Two", 2);
        let defender = Card::new("Ace", 14);
        let initial_p2_happiness = engine.players[1].happiness;
        apply_battle_result(&mut engine, &attacker, &defender);
        assert_eq!(engine.players[1].happiness, initial_p2_happiness + 5);
    }

    #[test]
    fn apply_battle_result_draw_no_happiness_change() {
        let mut engine = make_engine_with_hands();
        engine.draw_card(Card::new("Seven", 7)).unwrap();
        engine.play_card(0).unwrap();

        let card = Card::new("Seven", 7);
        let p0_before = engine.players[0].happiness;
        let p1_before = engine.players[1].happiness;
        apply_battle_result(&mut engine, &card, &card);
        assert_eq!(engine.players[0].happiness, p0_before);
        assert_eq!(engine.players[1].happiness, p1_before);
    }

    // ── spawn / despawn game UI ───────────────────────────────────────────────

    #[test]
    fn spawn_game_ui_creates_exactly_one_root() {
        let mut app = make_test_app();
        app.world_mut().run_system_once(spawn_game_ui);

        let count = app
            .world_mut()
            .query::<&GameUiRoot>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn spawn_game_ui_creates_hand_display_text() {
        let mut app = make_test_app();
        app.world_mut().run_system_once(spawn_game_ui);

        let count = app
            .world_mut()
            .query::<&HandDisplayText>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "should spawn exactly one HandDisplayText node");
    }

    #[test]
    fn spawn_game_ui_creates_instructions_text() {
        let mut app = make_test_app();
        app.world_mut().run_system_once(spawn_game_ui);

        let count = app
            .world_mut()
            .query::<&InstructionsText>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "should spawn exactly one InstructionsText node");
    }

    #[test]
    fn despawn_game_ui_removes_hand_display_text() {
        let mut app = make_test_app();
        app.world_mut().run_system_once(spawn_game_ui);
        app.world_mut().run_system_once(despawn_game_ui);

        let count = app
            .world_mut()
            .query::<&HandDisplayText>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0, "hand display should be removed on exit");
    }

    // ── ActionDeckResource ────────────────────────────────────────────────────

    #[test]
    fn action_deck_resource_wraps_deck() {
        let mut deck = Deck::new();
        deck.add_card(Card::new("Ace", 14));
        let res = ActionDeckResource(deck);
        assert_eq!(res.0.remaining(), 1);
    }

    // ── AttackerCardResource ──────────────────────────────────────────────────

    #[test]
    fn attacker_card_resource_stores_card() {
        let card = Card::new("King", 13);
        let res = AttackerCardResource(card.clone());
        assert_eq!(res.0.name, card.name);
        assert_eq!(res.0.value, card.value);
    }

    // ── AiOpponent ────────────────────────────────────────────────────────────

    #[test]
    fn setup_inserts_ai_opponent_resource() {
        let mut app = make_test_app();
        app.world_mut().run_system_once(setup);
        assert!(
            app.world().get_resource::<AiOpponent>().is_some(),
            "setup should insert AiOpponent resource"
        );
    }

    #[test]
    fn setup_names_player_two_cpu() {
        let mut app = make_test_app();
        app.world_mut().run_system_once(setup);
        let engine = app
            .world()
            .get_resource::<RulesEngineResource>()
            .expect("RulesEngineResource should be present after setup");
        assert_eq!(engine.0.players[1].name, "CPU");
    }
}
