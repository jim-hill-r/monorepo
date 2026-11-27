use bevy::prelude::*;
use rand::SeedableRng;
use rand::rngs::SmallRng;

use crate::ai::{AiOpponent, choose_attack_card, choose_defense_card};
use crate::card::Card;
use crate::card_library::{CardLibrary, default_action_deck};
use crate::deck::Deck;
use crate::input::{
    ActionButton, HAND_SLOTS, PlayerAction, send_button_actions, send_keyboard_actions,
};
use crate::rules::{Player, RulesEngine, TurnPhase};
use crate::state::AppState;
use crate::theme::{self, GameTheme};

/// Plugin that sets up the core game systems.
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerAction>()
            .add_systems(OnEnter(AppState::InGame), (setup, spawn_game_ui))
            .add_systems(
                Update,
                (
                    // Input first: keys and on-screen buttons become
                    // PlayerAction events, which handle_player_actions applies
                    // to RulesEngineResource (inserting or removing
                    // AttackerCardResource via Commands). apply_deferred
                    // flushes those commands so the UI systems below read the
                    // latest state.
                    (
                        send_keyboard_actions,
                        send_button_actions,
                        handle_player_actions,
                        // AI handlers run after the human handler so human
                        // input is processed first.  The AI acts automatically
                        // when it is Player 2's turn or when it needs to
                        // defend.
                        ai_handle_draw_phase,
                        ai_handle_play_phase,
                        ai_handle_battle_phase,
                        ai_handle_end_phase,
                    )
                        .chain(),
                    apply_deferred,
                    // UI systems run after deferred commands are applied.
                    (update_game_ui, update_pvp_ui, update_action_buttons).chain(),
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

/// Seeds the action deck's shuffle, so a game deals the same cards every
/// time. Scripted runs ([`crate::script`]) insert it; without it each game
/// shuffles at random.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShuffleSeed(pub u64);

/// Marks the root of the game UI so it can be cleaned up on exit.
#[derive(Component, Debug)]
pub struct GameUiRoot;

/// A per-player statistic shown in the game UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerStat {
    /// Remaining life, shown with the player's name.
    Life,
    /// Remaining civilians.
    Civilians,
    /// Current happiness score.
    Happiness,
}

/// Marks a text node that shows one statistic of one player.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerStatText {
    /// Index of the player in [`RulesEngine::players`].
    pub player: usize,
    /// The statistic this node shows.
    pub stat: PlayerStat,
}

impl PlayerStatText {
    /// Returns the text this node shows for `player`.
    pub fn text(&self, player: &Player) -> String {
        match self.stat {
            PlayerStat::Life => format!("{}: {} Life", player.name, player.life),
            PlayerStat::Civilians => format!("  Civilians: {}", player.civilians),
            PlayerStat::Happiness => format!("  Happiness: {}", player.happiness),
        }
    }
}

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

fn setup(mut commands: Commands, seed: Option<Res<ShuffleSeed>>) {
    let library = CardLibrary::default_library();
    info!(
        "Card library contains {} cards: {}",
        library.len(),
        library.card_names().join(", ")
    );

    // The unit tests in `card_library` guarantee the default deck builds, so
    // this error path only catches a broken card list during development.
    let mut action_deck: Deck = match default_action_deck(&library) {
        Ok(deck) => deck,
        Err(err) => {
            error!("Could not build the action deck: {err}");
            return;
        }
    };
    match seed {
        Some(seed) => action_deck.shuffle_with(&mut SmallRng::seed_from_u64(seed.0)),
        None => action_deck.shuffle(),
    }

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

/// What [`apply_player_action`] did, so the calling system can mirror it in
/// the ECS resources.
#[derive(Debug, Clone, PartialEq)]
pub enum ActionOutcome {
    /// The action doesn't apply now (wrong phase, the CPU's turn, no such
    /// card, or the game is over), so nothing changed.
    Ignored,
    /// The active player drew a card.
    Drew,
    /// The deck was empty when the player tried to draw, so the game ended.
    DeckExhausted,
    /// The active player played this card to attack.
    Attacked(Card),
    /// The defender picked a card and the battle was resolved.
    Defended,
    /// The active player ended the turn.
    TurnEnded,
}

/// Returns the index of the player who acts in the current phase: the
/// defender in the Battle phase, the active player otherwise.
pub fn acting_player(engine: &RulesEngine) -> usize {
    match engine.phase {
        TurnPhase::Battle => engine.inactive_player(),
        _ => engine.active_player,
    }
}

/// Returns `true` when the game is waiting on the human at this device.
///
/// In single-player mode (`ai_present`) the human is Player 1 (index 0), so
/// it is their move only when they are the acting player.  In hot-seat mode
/// both players share the device, so any open phase waits on the human.
pub fn awaits_human(engine: &RulesEngine, ai_present: bool) -> bool {
    !engine.is_game_over() && (!ai_present || acting_player(engine) == 0)
}

/// Applies one [`PlayerAction`] from the human to the game.
///
/// This is the one place keyboard and touch input change the game:
///
/// - **Advance** draws a card in the Draw phase (or ends the game when the
///   deck is empty) and ends the turn in the End phase.
/// - **PickCard(i)** plays card `i` to attack in the Play phase, or defends
///   with card `i` against `attacker` in the Battle phase.
///
/// Anything else, including any action while it is the CPU's move, is
/// [`ActionOutcome::Ignored`].
pub fn apply_player_action(
    engine: &mut RulesEngine,
    deck: &mut Deck,
    attacker: Option<&Card>,
    ai_present: bool,
    action: PlayerAction,
) -> ActionOutcome {
    if !awaits_human(engine, ai_present) {
        return ActionOutcome::Ignored;
    }

    match (&engine.phase, action) {
        (TurnPhase::Draw, PlayerAction::Advance) => match deck.draw() {
            Some(card) => match engine.draw_card(card) {
                Ok(()) => ActionOutcome::Drew,
                Err(e) => {
                    warn!("draw_card failed: {e}");
                    ActionOutcome::Ignored
                }
            },
            None => {
                // Deck is empty: end the game by scoring on happiness.
                engine.signal_deck_exhausted();
                ActionOutcome::DeckExhausted
            }
        },
        (TurnPhase::Play, PlayerAction::PickCard(idx)) => {
            if idx >= engine.players[engine.active_player].hand.len() {
                return ActionOutcome::Ignored;
            }
            match engine.play_card(idx) {
                Ok(card) => {
                    info!(
                        "{} plays {} ({}, value: {})",
                        engine.players[engine.active_player].name,
                        card.name,
                        card.category.label(),
                        card.value
                    );
                    ActionOutcome::Attacked(card)
                }
                Err(e) => {
                    warn!("play_card failed: {e}");
                    ActionOutcome::Ignored
                }
            }
        }
        (TurnPhase::Battle, PlayerAction::PickCard(idx)) => {
            let Some(attacker) = attacker else {
                return ActionOutcome::Ignored;
            };
            let defender_idx = engine.inactive_player();
            let Some(defender_card) = engine.players[defender_idx].play_card(idx) else {
                return ActionOutcome::Ignored;
            };
            apply_battle_result(engine, attacker, &defender_card);
            ActionOutcome::Defended
        }
        (TurnPhase::End, PlayerAction::Advance) => match engine.end_turn() {
            Ok(()) => ActionOutcome::TurnEnded,
            Err(e) => {
                warn!("end_turn failed: {e}");
                ActionOutcome::Ignored
            }
        },
        _ => ActionOutcome::Ignored,
    }
}

/// Resolves the battle at once when the defender has no cards in hand: the
/// defender loses to `attacker`, as if they played a card of value `0`.
///
/// Returns `true` when it resolved the battle.  Both the human and the AI
/// battle handlers use it.
pub fn defend_with_empty_hand(engine: &mut RulesEngine, attacker: &Card) -> bool {
    if !engine.players[engine.inactive_player()].hand.is_empty() {
        return false;
    }
    apply_battle_result(engine, attacker, &Card::new(EMPTY_HAND_CARD, 0));
    true
}

/// Applies the human's [`PlayerAction`] events (from the keyboard and the
/// on-screen buttons) through [`apply_player_action`], and keeps
/// [`AttackerCardResource`] in step: inserted when the human attacks, removed
/// once a battle is resolved.
///
/// When the human must defend with an empty hand, the battle resolves
/// without input and the attacker wins.
fn handle_player_actions(
    mut actions: EventReader<PlayerAction>,
    mut commands: Commands,
    engine_res: Option<ResMut<RulesEngineResource>>,
    deck_res: Option<ResMut<ActionDeckResource>>,
    attacker_res: Option<Res<AttackerCardResource>>,
    ai: Option<Res<AiOpponent>>,
) {
    let (Some(mut engine_res), Some(mut deck_res)) = (engine_res, deck_res) else {
        actions.clear();
        return;
    };
    let engine = &mut engine_res.0;
    let ai_present = ai.is_some();
    let mut attacker = attacker_res.map(|res| res.0.clone());

    for action in actions.read() {
        match apply_player_action(
            engine,
            &mut deck_res.0,
            attacker.as_ref(),
            ai_present,
            *action,
        ) {
            ActionOutcome::Attacked(card) => {
                commands.insert_resource(AttackerCardResource(card.clone()));
                attacker = Some(card);
            }
            ActionOutcome::Defended => {
                commands.remove_resource::<AttackerCardResource>();
                attacker = None;
            }
            _ => {}
        }
    }

    if engine.phase == TurnPhase::Battle
        && awaits_human(engine, ai_present)
        && let Some(attacker) = attacker
        && defend_with_empty_hand(engine, &attacker)
    {
        commands.remove_resource::<AttackerCardResource>();
    }
}

/// Resolves the battle between `attacker` and `defender` and logs the outcome.
///
/// Called by the human and AI battle handlers after both cards have been
/// selected.  Damage and the winner's happiness reward are applied by
/// [`RulesEngine::resolve_battle`].
fn apply_battle_result(engine: &mut RulesEngine, attacker: &Card, defender: &Card) {
    match engine.resolve_battle(attacker, defender) {
        Ok(outcome) => info!("Battle outcome: {outcome:?}"),
        Err(e) => warn!("resolve_battle failed: {e}"),
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

    if defend_with_empty_hand(engine, &attacker_card) {
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
///
/// The layout is compact so it fits a phone in landscape (about 360 logical
/// pixels tall): both players' stats sit side by side, the hand is a row of
/// buttons, and the touch targets are at least 48 pixels tall.
///
/// Colours come from the gamey theme in the player's chosen scheme and mode
/// ([`GameTheme`]): the screen is `surface`, headings and the hand are
/// `text`, the phase line is `accent` and the reminders are `text-muted`.
fn spawn_game_ui(mut commands: Commands, theme: Res<GameTheme>) {
    let tokens = theme.tokens();
    let text = theme::color(tokens.text);
    let muted = theme::color(tokens.text_muted);
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
                    row_gap: Val::Px(8.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },
                background_color: BackgroundColor(theme::color(tokens.surface)),
                ..default()
            },
        ))
        .with_children(|root| {
            // Title
            root.spawn(TextBundle::from_section(
                "Controlled Chaos - Player vs CPU",
                TextStyle {
                    font_size: theme.font(22.0),
                    color: text,
                    ..default()
                },
            ));

            // Both players' stats, side by side.
            root.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(48.0),
                    ..default()
                },
                ..default()
            })
            .with_children(|row| {
                spawn_player_stats(row, &theme, 0, "Player 1");
                spawn_player_stats(row, &theme, 1, "Player 2");
            });

            // Game Phase Info
            root.spawn((
                GamePhaseText,
                TextBundle::from_section(
                    "Phase: Draw | Round: 1",
                    TextStyle {
                        font_size: theme.font(16.0),
                        color: theme::color(tokens.accent),
                        ..default()
                    },
                ),
            ));

            // Whose hand is shown
            root.spawn((
                HandDisplayText,
                TextBundle::from_section(
                    "Hand: (loading...)",
                    TextStyle {
                        font_size: theme.font(16.0),
                        color: text,
                        ..default()
                    },
                ),
            ));

            // The hand: one button per card, shown by update_action_buttons.
            root.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                    max_width: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            })
            .with_children(|hand| {
                for idx in 0..HAND_SLOTS {
                    spawn_action_button(hand, &theme, PlayerAction::PickCard(idx), Val::Px(150.0));
                }
            });

            // Dynamic instructions
            root.spawn((
                InstructionsText,
                TextBundle::from_section(
                    "Tap Draw card or press SPACE",
                    TextStyle {
                        font_size: theme.font(16.0),
                        color: text,
                        ..default()
                    },
                ),
            ));

            // Draw a card / end the turn.
            spawn_action_button(root, &theme, PlayerAction::Advance, Val::Px(180.0));

            // Win condition reminder
            root.spawn(TextBundle::from_section(
                "Win: Highest happiness when deck runs out.  Lose: Life or civilians reach 0.",
                TextStyle {
                    font_size: theme.font(12.0),
                    color: muted,
                    ..default()
                },
            ));
        });
}

/// Spawns a column with one player's life, civilians and happiness.
///
/// Life is in `text` and the others in `text-muted`. The players are told
/// apart by name and position, not colour.
fn spawn_player_stats(parent: &mut ChildBuilder, theme: &GameTheme, player: usize, name: &str) {
    let tokens = theme.tokens();
    let (text, muted) = (theme::color(tokens.text), theme::color(tokens.text_muted));
    parent
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .with_children(|column| {
            let lines = [
                (PlayerStat::Life, format!("{name}: 20 Life"), 20.0, text),
                (
                    PlayerStat::Civilians,
                    "  Civilians: 2".to_string(),
                    14.0,
                    muted,
                ),
                (
                    PlayerStat::Happiness,
                    "  Happiness: 50".to_string(),
                    14.0,
                    muted,
                ),
            ];
            for (stat, text, font_size, color) in lines {
                column.spawn((
                    PlayerStatText { player, stat },
                    TextBundle::from_section(
                        text,
                        TextStyle {
                            font_size: theme.font(font_size),
                            color,
                            ..default()
                        },
                    ),
                ));
            }
        });
}

/// Spawns a hidden [`ActionButton`] with an empty label.
/// [`update_action_buttons`] shows it and sets its label.
fn spawn_action_button(
    parent: &mut ChildBuilder,
    theme: &GameTheme,
    action: PlayerAction,
    width: Val,
) {
    let colors = theme.button();
    parent
        .spawn((
            ActionButton(action),
            ButtonBundle {
                style: Style {
                    display: Display::None,
                    width,
                    min_height: Val::Px(theme.touch_target()),
                    padding: UiRect::all(Val::Px(6.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: BackgroundColor(theme::color(colors.normal)),
                ..default()
            },
        ))
        .with_children(|button| {
            button.spawn(TextBundle::from_section(
                "",
                TextStyle {
                    font_size: theme.font(15.0),
                    color: theme::color(colors.label),
                    ..default()
                },
            ));
        });
}

/// Returns the label of the button for the hand card at `idx`, such as
/// `"[1] Riot - Attack (5)"`.  The number is the key that picks it.
pub fn card_button_label(idx: usize, card: &Card) -> String {
    format!(
        "[{}] {} - {} ({})",
        idx + 1,
        card.name,
        card.category.label(),
        card.value
    )
}

/// Returns the label of the "next" button, or `None` when it should be
/// hidden because [`PlayerAction::Advance`] does nothing now.
pub fn advance_button_label(engine: &RulesEngine, ai_present: bool) -> Option<&'static str> {
    if !awaits_human(engine, ai_present) {
        return None;
    }
    match engine.phase {
        TurnPhase::Draw => Some("Draw card"),
        TurnPhase::End => Some("End turn"),
        TurnPhase::Play | TurnPhase::Battle => None,
    }
}

/// Returns the hand shown on screen: the acting player's, or `None` when it
/// is the CPU's and so stays hidden.
fn shown_hand(engine: &RulesEngine, ai_present: bool) -> Option<&[Card]> {
    let idx = acting_player(engine);
    (!(ai_present && idx == 1)).then(|| engine.players[idx].hand.as_slice())
}

/// Shows, labels and colours the on-screen [`ActionButton`]s for the
/// current state: one button per card in the shown hand (live in the Play
/// and Battle phases), and the Draw card / End turn button.
fn update_action_buttons(
    theme: Res<GameTheme>,
    engine_res: Option<Res<RulesEngineResource>>,
    ai: Option<Res<AiOpponent>>,
    mut buttons: Query<(
        &ActionButton,
        &Interaction,
        &mut Style,
        &mut BackgroundColor,
        &Children,
    )>,
    mut labels: Query<&mut Text>,
) {
    let Some(res) = engine_res else { return };
    let engine = &res.0;
    let ai_present = ai.is_some();
    let picking = awaits_human(engine, ai_present)
        && matches!(engine.phase, TurnPhase::Play | TurnPhase::Battle);
    let hand = shown_hand(engine, ai_present);
    let colors = theme.button();

    for (button, interaction, mut style, mut bg, children) in &mut buttons {
        let (label, live) = match button.0 {
            PlayerAction::PickCard(idx) => (
                hand.and_then(|hand| hand.get(idx))
                    .map(|card| card_button_label(idx, card)),
                picking,
            ),
            PlayerAction::Advance => (
                advance_button_label(engine, ai_present).map(str::to_string),
                true,
            ),
        };

        let Some(label) = label else {
            style.display = Display::None;
            continue;
        };
        style.display = Display::Flex;
        let (fill, ink) = colors.state(live, *interaction);
        *bg = BackgroundColor(theme::color(fill));
        for &child in children {
            if let Ok(mut text) = labels.get_mut(child) {
                if text.sections[0].value != label {
                    text.sections[0].value.clone_from(&label);
                }
                let ink = theme::color(ink);
                if text.sections[0].style.color != ink {
                    text.sections[0].style.color = ink;
                }
            }
        }
    }
}

/// Updates the game UI to reflect the current state of the rules engine.
fn update_game_ui(
    engine_res: Option<Res<RulesEngineResource>>,
    mut stat_query: Query<(&PlayerStatText, &mut Text), Without<GamePhaseText>>,
    mut phase_query: Query<&mut Text, (With<GamePhaseText>, Without<PlayerStatText>)>,
) {
    let Some(res) = engine_res else { return };
    let engine = &res.0;

    for (stat_text, mut text) in &mut stat_query {
        if let Some(player) = engine.players.get(stat_text.player) {
            text.sections[0].value = stat_text.text(player);
        }
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

    if let Ok(mut text) = hand_query.get_single_mut() {
        text.sections[0].value = hand_text(engine, is_ai_present);
    }

    if let Ok(mut text) = instr_query.get_single_mut() {
        text.sections[0].value = instructions_text(
            engine,
            deck_remaining,
            attacker_res.as_ref().map(|r| &r.0),
            is_ai_present,
        );
    }
}

/// Returns the label shown above the hand of cards.
///
/// During [`TurnPhase::Battle`] the defender's hand is shown so they can pick
/// a card to defend with; in every other phase it is the active player's. The
/// cards themselves are buttons (see `update_action_buttons`). When
/// `hide_ai_hand` is set (single-player mode), player two's hand is hidden.
fn hand_text(engine: &RulesEngine, hide_ai_hand: bool) -> String {
    let (idx, label) = match engine.phase {
        TurnPhase::Battle => {
            let def_idx = engine.inactive_player();
            (
                def_idx,
                format!("{}'s hand (defending):", engine.players[def_idx].name),
            )
        }
        _ => {
            let idx = engine.active_player;
            (idx, format!("{}'s hand:", engine.players[idx].name))
        }
    };

    if hide_ai_hand && idx == 1 {
        format!("{label} (hidden)")
    } else if engine.players[idx].hand.is_empty() {
        format!("{label} (empty)")
    } else {
        label
    }
}

/// Returns the instruction line for the current game state: the game-over
/// result, "CPU is taking its turn..." while the AI acts, or what the human
/// player should do in this phase.
fn instructions_text(
    engine: &RulesEngine,
    deck_remaining: usize,
    attacker: Option<&Card>,
    ai_present: bool,
) -> String {
    if engine.is_game_over() {
        return game_over_text(engine);
    }

    let ai_is_acting = ai_present
        && match engine.phase {
            TurnPhase::Battle => engine.inactive_player() == 1,
            _ => engine.active_player == 1,
        };
    if ai_is_acting {
        return "CPU is taking its turn...".to_string();
    }

    let active = &engine.players[engine.active_player];
    match engine.phase {
        TurnPhase::Draw => format!(
            "{}: Tap Draw card or press SPACE  ({} cards in deck)",
            active.name, deck_remaining
        ),
        TurnPhase::Play => format!(
            "{}: Tap a card or press 1-{} to play it",
            active.name,
            active.hand.len()
        ),
        TurnPhase::Battle => {
            let defender = &engine.players[engine.inactive_player()];
            let attacker_info = attacker
                .map(|c| format!("{} ({})", c.name, c.value))
                .unwrap_or_default();
            if defender.hand.is_empty() {
                format!(
                    "{} attacks with {}. {} has no cards - attacker wins!",
                    active.name, attacker_info, defender.name
                )
            } else {
                format!(
                    "{} attacks with {}.  {}: Tap a card or press 1-{} to defend",
                    active.name,
                    attacker_info,
                    defender.name,
                    defender.hand.len()
                )
            }
        }
        TurnPhase::End => format!(
            "Turn over.  Tap End turn or press SPACE to pass to {}",
            engine.players[engine.inactive_player()].name
        ),
    }
}

/// Returns the game-over message: the winner and why, or a draw.
fn game_over_text(engine: &RulesEngine) -> String {
    let Some(idx) = engine.winner() else {
        return "GAME OVER - It's a draw!".to_string();
    };
    let winner = &engine.players[idx];
    let loser = &engine.players[1 - idx];
    if loser.is_alive() && engine.deck_exhausted {
        // Deck ran out: the winner has the higher happiness.
        format!(
            "GAME OVER - {} wins! (Happiness: {} vs {})",
            winner.name, winner.happiness, loser.happiness,
        )
    } else {
        // Immediate defeat: the loser lost all life or civilians.
        let reason = if loser.life == 0 {
            "ran out of life"
        } else {
            "lost all civilians"
        };
        format!(
            "GAME OVER - {} wins! ({} {})",
            winner.name, loser.name, reason
        )
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
        app.init_resource::<GameTheme>();
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
        assert_eq!(
            engine.players[0].happiness,
            initial_happiness + Player::BATTLE_WIN_HAPPINESS as u32
        );
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
        assert_eq!(
            engine.players[1].happiness,
            initial_p2_happiness + Player::BATTLE_WIN_HAPPINESS as u32
        );
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

    // ── PlayerStatText / update_game_ui ──────────────────────────────────────

    #[test]
    fn player_stat_text_formats_each_stat() {
        let mut player = Player::new("Alice", 17);
        player.civilians = 3;
        player.happiness = 42;
        let text = |stat| PlayerStatText { player: 0, stat }.text(&player);
        assert_eq!(text(PlayerStat::Life), "Alice: 17 Life");
        assert_eq!(text(PlayerStat::Civilians), "  Civilians: 3");
        assert_eq!(text(PlayerStat::Happiness), "  Happiness: 42");
    }

    #[test]
    fn spawn_game_ui_creates_one_text_per_player_stat() {
        let mut app = make_test_app();
        app.world_mut().run_system_once(spawn_game_ui);

        let mut markers: Vec<(usize, PlayerStat)> = app
            .world_mut()
            .query::<&PlayerStatText>()
            .iter(app.world())
            .map(|m| (m.player, m.stat))
            .collect();
        markers.sort_by_key(|&(player, stat)| (player, stat as u8));
        let mut expected = Vec::new();
        for player in 0..2 {
            for stat in [
                PlayerStat::Life,
                PlayerStat::Civilians,
                PlayerStat::Happiness,
            ] {
                expected.push((player, stat));
            }
        }
        assert_eq!(markers, expected);
    }

    /// Spawns the UI, applies `change` to the engine, runs `update_game_ui`
    /// and returns the text of every player stat node.
    fn player_stat_texts_after(
        change: impl FnOnce(&mut RulesEngine),
    ) -> Vec<(PlayerStatText, String)> {
        let mut app = make_test_app();
        let mut engine = make_engine_with_hands();
        change(&mut engine);
        app.insert_resource(RulesEngineResource(engine));
        app.world_mut().run_system_once(spawn_game_ui);
        app.world_mut().run_system_once(update_game_ui);

        let mut texts: Vec<(PlayerStatText, String)> = app
            .world_mut()
            .query::<(&PlayerStatText, &Text)>()
            .iter(app.world())
            .map(|(m, t)| (*m, t.sections[0].value.clone()))
            .collect();
        texts.sort_by_key(|(m, _)| (m.player, m.stat as u8));
        texts
    }

    #[test]
    fn update_game_ui_shows_each_players_stats() {
        let texts = player_stat_texts_after(|engine| {
            engine.players[0].life = 15;
            engine.players[0].civilians = 1;
            engine.players[0].happiness = 60;
            engine.players[1].life = 9;
            engine.players[1].civilians = 4;
            engine.players[1].happiness = 35;
        });
        let values: Vec<&str> = texts.iter().map(|(_, t)| t.as_str()).collect();
        assert_eq!(
            values,
            [
                "Alice: 15 Life",
                "  Civilians: 1",
                "  Happiness: 60",
                "Bob: 9 Life",
                "  Civilians: 4",
                "  Happiness: 35",
            ]
        );
    }

    #[test]
    fn update_game_ui_updates_phase_text() {
        let mut app = make_test_app();
        app.insert_resource(RulesEngineResource(make_engine_with_hands()));
        app.world_mut().run_system_once(spawn_game_ui);
        app.world_mut().run_system_once(update_game_ui);

        let text = app
            .world_mut()
            .query_filtered::<&Text, With<GamePhaseText>>()
            .single(app.world())
            .sections[0]
            .value
            .clone();
        assert_eq!(text, "Phase: Draw | Round: 1 | Active: Alice");
    }

    #[test]
    fn update_game_ui_without_engine_leaves_text_unchanged() {
        let mut app = make_test_app();
        app.world_mut().run_system_once(spawn_game_ui);
        app.world_mut().run_system_once(update_game_ui);

        let life = app
            .world_mut()
            .query::<(&PlayerStatText, &Text)>()
            .iter(app.world())
            .find(|(m, _)| m.player == 0 && m.stat == PlayerStat::Life)
            .map(|(_, t)| t.sections[0].value.clone());
        assert_eq!(life.as_deref(), Some("Player 1: 20 Life"));
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

    fn dealt_hands(seed: Option<u64>) -> Vec<Vec<String>> {
        let mut app = make_test_app();
        if let Some(seed) = seed {
            app.insert_resource(ShuffleSeed(seed));
        }
        app.world_mut().run_system_once(setup);
        let engine = &app.world().resource::<RulesEngineResource>().0;
        engine
            .players
            .iter()
            .map(|p| p.hand.iter().map(|c| c.name.clone()).collect())
            .collect()
    }

    #[test]
    fn setup_with_a_seed_deals_the_same_hands_every_time() {
        assert_eq!(dealt_hands(Some(7)), dealt_hands(Some(7)));
    }

    #[test]
    fn setup_with_different_seeds_deals_different_hands() {
        assert_ne!(dealt_hands(Some(7)), dealt_hands(Some(8)));
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

    // ── apply_player_action ───────────────────────────────────────────────────

    fn deck_of(values: &[u32]) -> Deck {
        let mut deck = Deck::new();
        for (i, value) in values.iter().enumerate() {
            deck.add_card(Card::new(format!("Card {i}"), *value));
        }
        deck
    }

    #[test]
    fn advance_in_draw_phase_draws_a_card() {
        let mut engine = make_engine_with_hands();
        let mut deck = deck_of(&[9]);
        let outcome =
            apply_player_action(&mut engine, &mut deck, None, true, PlayerAction::Advance);
        assert_eq!(outcome, ActionOutcome::Drew);
        assert_eq!(engine.phase, TurnPhase::Play);
        assert_eq!(engine.players[0].hand.len(), 3);
        assert_eq!(deck.remaining(), 0);
    }

    #[test]
    fn advance_with_empty_deck_ends_the_game() {
        let mut engine = make_engine_with_hands();
        let outcome = apply_player_action(
            &mut engine,
            &mut Deck::new(),
            None,
            true,
            PlayerAction::Advance,
        );
        assert_eq!(outcome, ActionOutcome::DeckExhausted);
        assert!(engine.is_game_over());
    }

    #[test]
    fn pick_card_in_draw_phase_is_ignored() {
        let mut engine = make_engine_with_hands();
        let mut deck = deck_of(&[9]);
        let outcome = apply_player_action(
            &mut engine,
            &mut deck,
            None,
            true,
            PlayerAction::PickCard(0),
        );
        assert_eq!(outcome, ActionOutcome::Ignored);
        assert_eq!(engine.phase, TurnPhase::Draw);
        assert_eq!(deck.remaining(), 1);
    }

    #[test]
    fn pick_card_in_play_phase_attacks_with_it() {
        let mut engine = make_engine_with_hands();
        engine.draw_card(Card::new("Two", 2)).unwrap();
        let outcome = apply_player_action(
            &mut engine,
            &mut Deck::new(),
            None,
            true,
            PlayerAction::PickCard(1),
        );
        assert_eq!(outcome, ActionOutcome::Attacked(Card::new("King", 13)));
        assert_eq!(engine.phase, TurnPhase::Battle);
        assert_eq!(engine.players[0].hand.len(), 2);
    }

    #[test]
    fn pick_card_past_the_hand_is_ignored() {
        let mut engine = make_engine_with_hands();
        engine.draw_card(Card::new("Two", 2)).unwrap();
        let outcome = apply_player_action(
            &mut engine,
            &mut Deck::new(),
            None,
            true,
            PlayerAction::PickCard(3),
        );
        assert_eq!(outcome, ActionOutcome::Ignored);
        assert_eq!(engine.phase, TurnPhase::Play);
    }

    #[test]
    fn advance_in_play_phase_is_ignored() {
        let mut engine = make_engine_with_hands();
        engine.draw_card(Card::new("Two", 2)).unwrap();
        let outcome = apply_player_action(
            &mut engine,
            &mut Deck::new(),
            None,
            true,
            PlayerAction::Advance,
        );
        assert_eq!(outcome, ActionOutcome::Ignored);
        assert_eq!(engine.phase, TurnPhase::Play);
    }

    /// An engine in the Battle phase where Bob (index 1) attacked Alice with
    /// a card of value 5, so Alice (the human in single-player mode) defends.
    fn engine_with_alice_defending() -> (RulesEngine, Card) {
        let mut engine = make_engine_with_hands();
        engine.active_player = 1;
        engine.draw_card(Card::new("Five", 5)).unwrap();
        let hand_len = engine.players[1].hand.len();
        let attacker = engine.play_card(hand_len - 1).unwrap();
        (engine, attacker)
    }

    #[test]
    fn pick_card_in_battle_phase_defends_with_it() {
        let (mut engine, attacker) = engine_with_alice_defending();
        let alice_happiness = engine.players[0].happiness;
        let outcome = apply_player_action(
            &mut engine,
            &mut Deck::new(),
            Some(&attacker),
            true,
            PlayerAction::PickCard(0),
        );
        assert_eq!(outcome, ActionOutcome::Defended);
        assert_eq!(engine.phase, TurnPhase::End);
        // Alice's Ace (14) beats the Five.
        assert_eq!(engine.players[0].hand.len(), 1);
        assert_eq!(
            engine.players[0].happiness,
            alice_happiness + Player::BATTLE_WIN_HAPPINESS as u32
        );
    }

    #[test]
    fn pick_card_in_battle_phase_without_attacker_is_ignored() {
        let (mut engine, _) = engine_with_alice_defending();
        let outcome = apply_player_action(
            &mut engine,
            &mut Deck::new(),
            None,
            true,
            PlayerAction::PickCard(0),
        );
        assert_eq!(outcome, ActionOutcome::Ignored);
        assert_eq!(engine.players[0].hand.len(), 2);
    }

    #[test]
    fn advance_in_end_phase_ends_the_turn() {
        let (mut engine, attacker) = engine_with_alice_defending();
        engine.active_player = 0;
        apply_battle_result(&mut engine, &attacker, &Card::new("Two", 2));
        assert_eq!(engine.phase, TurnPhase::End);
        let outcome = apply_player_action(
            &mut engine,
            &mut Deck::new(),
            None,
            true,
            PlayerAction::Advance,
        );
        assert_eq!(outcome, ActionOutcome::TurnEnded);
        assert_eq!(engine.phase, TurnPhase::Draw);
        assert_eq!(engine.active_player, 1);
    }

    #[test]
    fn actions_on_the_cpus_turn_are_ignored() {
        let mut engine = make_engine_with_hands();
        engine.active_player = 1;
        let mut deck = deck_of(&[9]);
        let outcome =
            apply_player_action(&mut engine, &mut deck, None, true, PlayerAction::Advance);
        assert_eq!(outcome, ActionOutcome::Ignored);
        assert_eq!(deck.remaining(), 1);
    }

    #[test]
    fn hot_seat_lets_either_player_act() {
        let mut engine = make_engine_with_hands();
        engine.active_player = 1;
        let outcome = apply_player_action(
            &mut engine,
            &mut deck_of(&[9]),
            None,
            false,
            PlayerAction::Advance,
        );
        assert_eq!(outcome, ActionOutcome::Drew);
    }

    #[test]
    fn actions_after_game_over_are_ignored() {
        let mut engine = make_engine_with_hands();
        engine.signal_deck_exhausted();
        let outcome = apply_player_action(
            &mut engine,
            &mut deck_of(&[9]),
            None,
            true,
            PlayerAction::Advance,
        );
        assert_eq!(outcome, ActionOutcome::Ignored);
    }

    // ── defend_with_empty_hand ────────────────────────────────────────────────

    #[test]
    fn defend_with_empty_hand_resolves_the_battle() {
        let (mut engine, attacker) = engine_with_alice_defending();
        engine.players[0].hand.clear();
        let bob_happiness = engine.players[1].happiness;
        assert!(defend_with_empty_hand(&mut engine, &attacker));
        assert_eq!(engine.phase, TurnPhase::End);
        assert_eq!(
            engine.players[1].happiness,
            bob_happiness + Player::BATTLE_WIN_HAPPINESS as u32
        );
    }

    #[test]
    fn defend_with_empty_hand_waits_when_defender_has_cards() {
        let (mut engine, attacker) = engine_with_alice_defending();
        assert!(!defend_with_empty_hand(&mut engine, &attacker));
        assert_eq!(engine.phase, TurnPhase::Battle);
    }

    // ── on-screen buttons ─────────────────────────────────────────────────────

    #[test]
    fn card_button_label_shows_key_name_category_and_value() {
        let card = Card::new("Ace", 14);
        assert_eq!(
            card_button_label(0, &card),
            format!("[1] Ace - {} (14)", card.category.label())
        );
    }

    #[test]
    fn advance_button_label_per_phase() {
        let mut engine = make_engine_with_hands();
        assert_eq!(advance_button_label(&engine, true), Some("Draw card"));
        engine.draw_card(Card::new("Two", 2)).unwrap();
        assert_eq!(advance_button_label(&engine, true), None, "Play phase");
        let attacker = engine.play_card(0).unwrap();
        assert_eq!(advance_button_label(&engine, true), None, "Battle phase");
        apply_battle_result(&mut engine, &attacker, &Card::new("Two", 2));
        assert_eq!(advance_button_label(&engine, true), Some("End turn"));
        engine.active_player = 1;
        assert_eq!(advance_button_label(&engine, true), None, "CPU's turn");
    }

    #[test]
    fn spawn_game_ui_creates_a_button_per_hand_slot_and_one_advance_button() {
        let mut app = make_test_app();
        app.world_mut().run_system_once(spawn_game_ui);
        let mut actions: Vec<PlayerAction> = app
            .world_mut()
            .query::<&ActionButton>()
            .iter(app.world())
            .map(|b| b.0)
            .collect();
        actions.sort_by_key(|a| match a {
            PlayerAction::PickCard(i) => *i,
            PlayerAction::Advance => usize::MAX,
        });
        let mut expected: Vec<PlayerAction> = (0..HAND_SLOTS).map(PlayerAction::PickCard).collect();
        expected.push(PlayerAction::Advance);
        assert_eq!(actions, expected);
    }

    /// Returns each visible button's action and label after one UI update.
    fn visible_buttons(engine: RulesEngine) -> Vec<(PlayerAction, String)> {
        let mut app = make_test_app();
        app.insert_resource(RulesEngineResource(engine))
            .insert_resource(AiOpponent);
        app.world_mut().run_system_once(spawn_game_ui);
        app.world_mut().run_system_once(update_action_buttons);
        let world = app.world_mut();
        let buttons: Vec<(PlayerAction, Display, Entity)> = world
            .query::<(&ActionButton, &Style, &Children)>()
            .iter(world)
            .map(|(b, s, c)| (b.0, s.display, c[0]))
            .collect();
        buttons
            .into_iter()
            .filter(|(_, display, _)| *display != Display::None)
            .map(|(action, _, label)| {
                let text = world
                    .get::<Text>(label)
                    .map(|t| t.sections[0].value.clone());
                (action, text.unwrap_or_default())
            })
            .collect()
    }

    #[test]
    fn update_action_buttons_shows_the_hand_and_draw_button() {
        let buttons = visible_buttons(make_engine_with_hands());
        let actions: Vec<PlayerAction> = buttons.iter().map(|(a, _)| *a).collect();
        assert_eq!(
            actions,
            [
                PlayerAction::PickCard(0),
                PlayerAction::PickCard(1),
                PlayerAction::Advance
            ]
        );
        assert!(buttons[0].1.starts_with("[1] Ace - "));
        assert_eq!(buttons[2].1, "Draw card");
    }

    #[test]
    fn update_action_buttons_hides_the_cpus_hand() {
        let mut engine = make_engine_with_hands();
        engine.active_player = 1;
        assert!(visible_buttons(engine).is_empty());
    }

    // ── handle_player_actions ─────────────────────────────────────────────────

    #[test]
    fn handle_player_actions_applies_a_tap() {
        let mut app = make_test_app();
        app.add_event::<PlayerAction>()
            .insert_resource(RulesEngineResource(make_engine_with_hands()))
            .insert_resource(ActionDeckResource(deck_of(&[9])))
            .insert_resource(AiOpponent);
        app.world_mut().send_event(PlayerAction::Advance);
        app.world_mut().run_system_once(handle_player_actions);
        app.world_mut().send_event(PlayerAction::PickCard(0));
        app.world_mut().run_system_once(handle_player_actions);

        let engine = &app.world().resource::<RulesEngineResource>().0;
        assert_eq!(engine.phase, TurnPhase::Battle);
        let attacker = app.world().get_resource::<AttackerCardResource>();
        assert_eq!(attacker.map(|a| a.0.name.as_str()), Some("Ace"));
    }

    // ── hand_text / instructions_text ─────────────────────────────────────────

    #[test]
    fn hand_text_shows_active_players_hand_in_draw_phase() {
        let engine = make_engine_with_hands();
        assert_eq!(hand_text(&engine, false), "Alice's hand:");
    }

    #[test]
    fn hand_text_shows_defenders_hand_in_battle_phase() {
        let mut engine = make_engine_with_hands();
        engine.draw_card(Card::new("Ten", 10)).unwrap();
        engine.play_card(0).unwrap();
        assert_eq!(hand_text(&engine, false), "Bob's hand (defending):");
    }

    #[test]
    fn hand_text_hides_ai_hand() {
        let mut engine = make_engine_with_hands();
        engine.draw_card(Card::new("Ten", 10)).unwrap();
        engine.play_card(0).unwrap();
        assert_eq!(hand_text(&engine, true), "Bob's hand (defending): (hidden)");
    }

    #[test]
    fn hand_text_does_not_hide_human_hand_when_ai_present() {
        let engine = make_engine_with_hands();
        assert_eq!(hand_text(&engine, true), "Alice's hand:");
    }

    #[test]
    fn hand_text_marks_empty_hand() {
        let engine = RulesEngine::new(Player::new("Alice", 20), Player::new("Bob", 20));
        assert_eq!(hand_text(&engine, false), "Alice's hand: (empty)");
    }

    #[test]
    fn instructions_deck_exhausted_with_winner() {
        let mut engine = make_engine_with_hands();
        engine.players[0].happiness = 60;
        engine.players[1].happiness = 40;
        engine.signal_deck_exhausted();
        assert_eq!(
            instructions_text(&engine, 0, None, false),
            "GAME OVER - Alice wins! (Happiness: 60 vs 40)"
        );
    }

    #[test]
    fn instructions_deck_exhausted_with_draw() {
        let mut engine = make_engine_with_hands();
        engine.signal_deck_exhausted();
        assert_eq!(
            instructions_text(&engine, 0, None, false),
            "GAME OVER - It's a draw!"
        );
    }

    #[test]
    fn instructions_life_reaching_zero() {
        let mut engine = make_engine_with_hands();
        engine.players[0].life = 0;
        assert_eq!(
            instructions_text(&engine, 5, None, false),
            "GAME OVER - Bob wins! (Alice ran out of life)"
        );
    }

    #[test]
    fn instructions_civilians_reaching_zero() {
        let mut engine = make_engine_with_hands();
        engine.players[1].civilians = 0;
        assert_eq!(
            instructions_text(&engine, 5, None, false),
            "GAME OVER - Alice wins! (Bob lost all civilians)"
        );
    }

    #[test]
    fn instructions_defeat_reason_wins_over_deck_exhaustion() {
        let mut engine = make_engine_with_hands();
        engine.players[1].life = 0;
        engine.players[1].happiness = 90;
        engine.signal_deck_exhausted();
        assert_eq!(
            instructions_text(&engine, 0, None, false),
            "GAME OVER - Alice wins! (Bob ran out of life)"
        );
    }

    #[test]
    fn instructions_game_over_takes_precedence_over_cpu_turn() {
        let mut engine = make_engine_with_hands();
        engine.draw_card(Card::new("Ten", 10)).unwrap();
        engine.play_card(0).unwrap();
        engine.players[0].life = 0;
        assert_eq!(
            instructions_text(&engine, 5, None, true),
            "GAME OVER - Bob wins! (Alice ran out of life)"
        );
    }

    #[test]
    fn instructions_cpu_taking_its_turn_when_ai_active() {
        let mut engine = make_engine_with_hands();
        engine.active_player = 1;
        assert_eq!(
            instructions_text(&engine, 5, None, true),
            "CPU is taking its turn..."
        );
    }

    #[test]
    fn instructions_cpu_taking_its_turn_when_ai_defends() {
        let mut engine = make_engine_with_hands();
        engine.draw_card(Card::new("Ten", 10)).unwrap();
        let attacker = engine.play_card(0).unwrap();
        assert_eq!(
            instructions_text(&engine, 5, Some(&attacker), true),
            "CPU is taking its turn..."
        );
    }

    #[test]
    fn instructions_draw_phase() {
        let engine = make_engine_with_hands();
        assert_eq!(
            instructions_text(&engine, 7, None, true),
            "Alice: Tap Draw card or press SPACE  (7 cards in deck)"
        );
    }

    #[test]
    fn instructions_play_phase() {
        let mut engine = make_engine_with_hands();
        engine.draw_card(Card::new("Ten", 10)).unwrap();
        assert_eq!(
            instructions_text(&engine, 6, None, false),
            "Alice: Tap a card or press 1-3 to play it"
        );
    }

    #[test]
    fn instructions_battle_phase_human_defender() {
        let mut engine = make_engine_with_hands();
        engine.draw_card(Card::new("Ten", 10)).unwrap();
        let attacker = engine.play_card(0).unwrap();
        assert_eq!(
            instructions_text(&engine, 6, Some(&attacker), false),
            format!(
                "Alice attacks with {} ({}).  Bob: Tap a card or press 1-2 to defend",
                attacker.name, attacker.value
            )
        );
    }

    #[test]
    fn instructions_battle_phase_defender_without_cards() {
        let mut engine = make_engine_with_hands();
        engine.players[1].hand.clear();
        engine.draw_card(Card::new("Ten", 10)).unwrap();
        let attacker = engine.play_card(0).unwrap();
        assert_eq!(
            instructions_text(&engine, 6, Some(&attacker), false),
            format!(
                "Alice attacks with {} ({}). Bob has no cards - attacker wins!",
                attacker.name, attacker.value
            )
        );
    }
}
