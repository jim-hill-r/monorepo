use crate::card::Card;

/// The phase of the current turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnPhase {
    /// Active player draws a card from their deck into their hand.
    Draw,
    /// Active player selects a card from their hand to play as the attacker.
    Play,
    /// Cards are compared and damage is applied to the loser.
    Battle,
    /// Turn is over; control passes to the next player.
    End,
}

/// Outcome of a single battle between two played cards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattleOutcome {
    /// Attacker's card had a higher value; the defender takes 1 damage.
    AttackerWins,
    /// Defender's card had a higher value; the attacker takes 1 damage.
    DefenderWins,
    /// Both cards had equal value; no damage is applied.
    Draw,
}

/// A player in the game.
#[derive(Debug, Clone)]
pub struct Player {
    /// Display name of the player.
    pub name: String,
    /// Cards currently held in hand.
    pub hand: Vec<Card>,
    /// Remaining life points.
    pub life: u32,
    /// Number of civilian cards in this player's community.
    ///
    /// Per the rulebook each player starts with two civilians.  A player
    /// automatically loses when this reaches zero.
    pub civilians: u32,
    /// Happiness score of this player's community (0–100).
    ///
    /// The winning player is the one with the highest happiness score at game
    /// end.  Some actions raise or lower happiness.
    pub happiness: u32,
}

impl Player {
    /// Initial number of civilians per player as defined by the rulebook.
    pub const STARTING_CIVILIANS: u32 = 2;

    /// Initial happiness score per player.
    pub const STARTING_HAPPINESS: u32 = 50;

    /// Creates a new player with an empty hand and the given starting life total.
    ///
    /// Civilians are initialized to [`STARTING_CIVILIANS`] and happiness to
    /// [`STARTING_HAPPINESS`] as specified by the rulebook.
    ///
    /// [`STARTING_CIVILIANS`]: Player::STARTING_CIVILIANS
    /// [`STARTING_HAPPINESS`]: Player::STARTING_HAPPINESS
    pub fn new(name: impl Into<String>, starting_life: u32) -> Self {
        Self {
            name: name.into(),
            hand: Vec::new(),
            life: starting_life,
            civilians: Self::STARTING_CIVILIANS,
            happiness: Self::STARTING_HAPPINESS,
        }
    }

    /// Returns `true` if the player still has at least one life point **and**
    /// at least one civilian.
    ///
    /// Per the rulebook a player automatically loses when they have no
    /// civilians left.
    pub fn is_alive(&self) -> bool {
        self.life > 0 && self.has_civilians()
    }

    /// Returns `true` when this player still has at least one civilian.
    pub fn has_civilians(&self) -> bool {
        self.civilians > 0
    }

    /// Adjusts the happiness score by `delta`, clamping the result to `[0, 100]`.
    pub fn adjust_happiness(&mut self, delta: i32) {
        let new_val = self.happiness as i32 + delta;
        self.happiness = new_val.clamp(0, 100) as u32;
    }

    /// Adds a card to the player's hand.
    pub fn receive_card(&mut self, card: Card) {
        self.hand.push(card);
    }

    /// Removes and returns the card at `index` from the player's hand.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn play_card(&mut self, index: usize) -> Option<Card> {
        if index < self.hand.len() {
            Some(self.hand.remove(index))
        } else {
            None
        }
    }
}

/// Errors that can occur when interacting with the [`RulesEngine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RulesError {
    /// An action was attempted during the wrong phase.
    WrongPhase {
        expected: TurnPhase,
        actual: TurnPhase,
    },
    /// A hand index was provided that does not correspond to any card.
    InvalidCardIndex(usize),
}

impl std::fmt::Display for RulesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RulesError::WrongPhase { expected, actual } => {
                write!(f, "wrong phase: expected {expected:?}, got {actual:?}")
            }
            RulesError::InvalidCardIndex(i) => {
                write!(f, "no card at hand index {i}")
            }
        }
    }
}

/// Central rules engine for the two-player turn-based card game.
///
/// # Turn flow
///
/// Each turn follows four phases in order:
/// 1. **Draw** – call [`draw_card`] to deal the active player a card.
/// 2. **Play** – call [`play_card`] to have the active player choose a card from hand.
/// 3. **Battle** – call [`resolve_battle`] with both players' chosen cards to apply damage.
/// 4. **End** – call [`end_turn`] to pass control to the other player and start the next round.
///
/// # Win / loss conditions
///
/// - A player **loses immediately** when they have no civilians remaining.
/// - When the action deck runs out (signalled via [`signal_deck_exhausted`]), the game
///   ends and the player with the **higher happiness score wins**.  A tie is possible.
///
/// [`draw_card`]: RulesEngine::draw_card
/// [`play_card`]: RulesEngine::play_card
/// [`resolve_battle`]: RulesEngine::resolve_battle
/// [`end_turn`]: RulesEngine::end_turn
/// [`signal_deck_exhausted`]: RulesEngine::signal_deck_exhausted
#[derive(Debug)]
pub struct RulesEngine {
    /// The two players; index `0` is player one, index `1` is player two.
    pub players: [Player; 2],
    /// Index of the player whose turn it currently is (`0` or `1`).
    pub active_player: usize,
    /// Current phase within the active player's turn.
    pub phase: TurnPhase,
    /// Current round number, starting at `1` and incrementing after each
    /// [`end_turn`] call.
    ///
    /// [`end_turn`]: RulesEngine::end_turn
    pub round: u32,
    /// Set to `true` once the caller signals that the action deck is exhausted.
    ///
    /// When `true`, [`is_game_over`] returns `true` and [`winner`] resolves
    /// the result by comparing happiness scores.
    ///
    /// [`is_game_over`]: RulesEngine::is_game_over
    /// [`winner`]: RulesEngine::winner
    pub deck_exhausted: bool,
}

impl RulesEngine {
    /// Creates a new engine with two players and sets the first player as active.
    pub fn new(player_one: Player, player_two: Player) -> Self {
        Self {
            players: [player_one, player_two],
            active_player: 0,
            phase: TurnPhase::Draw,
            round: 1,
            deck_exhausted: false,
        }
    }

    /// Returns the index of the inactive (defending) player.
    pub fn inactive_player(&self) -> usize {
        1 - self.active_player
    }

    /// Deals `card` to the active player's hand.
    ///
    /// May only be called during the [`Draw`] phase.
    /// On success the phase advances to [`Play`].
    ///
    /// [`Draw`]: TurnPhase::Draw
    /// [`Play`]: TurnPhase::Play
    pub fn draw_card(&mut self, card: Card) -> Result<(), RulesError> {
        self.require_phase(&TurnPhase::Draw)?;
        self.players[self.active_player].receive_card(card);
        self.phase = TurnPhase::Play;
        Ok(())
    }

    /// Active player plays the card at `hand_index`.
    ///
    /// May only be called during the [`Play`] phase.
    /// On success the removed card is returned and the phase advances to [`Battle`].
    ///
    /// [`Play`]: TurnPhase::Play
    /// [`Battle`]: TurnPhase::Battle
    pub fn play_card(&mut self, hand_index: usize) -> Result<Card, RulesError> {
        self.require_phase(&TurnPhase::Play)?;
        let card = self.players[self.active_player]
            .play_card(hand_index)
            .ok_or(RulesError::InvalidCardIndex(hand_index))?;
        self.phase = TurnPhase::Battle;
        Ok(card)
    }

    /// Compares `attacker` (active player's card) against `defender` (inactive player's card).
    ///
    /// The losing player takes 1 point of damage.  On a draw neither player is damaged.
    /// May only be called during the [`Battle`] phase.
    /// On success the phase advances to [`End`].
    ///
    /// [`Battle`]: TurnPhase::Battle
    /// [`End`]: TurnPhase::End
    pub fn resolve_battle(
        &mut self,
        attacker: &Card,
        defender: &Card,
    ) -> Result<BattleOutcome, RulesError> {
        self.require_phase(&TurnPhase::Battle)?;
        let outcome = if attacker.value > defender.value {
            BattleOutcome::AttackerWins
        } else if defender.value > attacker.value {
            BattleOutcome::DefenderWins
        } else {
            BattleOutcome::Draw
        };

        match &outcome {
            BattleOutcome::AttackerWins => {
                let defender_idx = self.inactive_player();
                self.players[defender_idx].life = self.players[defender_idx].life.saturating_sub(1);
            }
            BattleOutcome::DefenderWins => {
                self.players[self.active_player].life =
                    self.players[self.active_player].life.saturating_sub(1);
            }
            BattleOutcome::Draw => {}
        }

        self.phase = TurnPhase::End;
        Ok(outcome)
    }

    /// Ends the current turn and passes control to the other player.
    ///
    /// May only be called during the [`End`] phase.
    /// Increments the round counter and resets the phase to [`Draw`].
    ///
    /// [`End`]: TurnPhase::End
    /// [`Draw`]: TurnPhase::Draw
    pub fn end_turn(&mut self) -> Result<(), RulesError> {
        self.require_phase(&TurnPhase::End)?;
        self.active_player = self.inactive_player();
        self.round += 1;
        self.phase = TurnPhase::Draw;
        Ok(())
    }

    /// Notifies the engine that the action deck has been exhausted.
    ///
    /// Once called, [`is_game_over`] returns `true` and [`winner`] determines
    /// the result by comparing happiness scores.
    ///
    /// [`is_game_over`]: RulesEngine::is_game_over
    /// [`winner`]: RulesEngine::winner
    pub fn signal_deck_exhausted(&mut self) {
        self.deck_exhausted = true;
    }

    /// Returns the index of the winning player if the game is over, otherwise `None`.
    ///
    /// Win conditions (checked in order):
    /// 1. A player who is no longer alive (see [`Player::is_alive`]) — i.e. their
    ///    life points **or** civilian count has reached zero — loses immediately
    ///    and the opponent wins.
    /// 2. When the action deck is exhausted (see [`signal_deck_exhausted`]), the
    ///    player with the higher happiness score wins.  If both players have the
    ///    same happiness the result is `None` (a draw).
    ///
    /// [`Player::is_alive`]: Player::is_alive
    /// [`signal_deck_exhausted`]: RulesEngine::signal_deck_exhausted
    pub fn winner(&self) -> Option<usize> {
        // Immediate loss: a player who is no longer alive loses.
        if !self.players[0].is_alive() {
            return Some(1);
        }
        if !self.players[1].is_alive() {
            return Some(0);
        }

        // Deck-exhaustion victory: highest happiness wins.
        if self.deck_exhausted {
            return match self.players[0].happiness.cmp(&self.players[1].happiness) {
                std::cmp::Ordering::Greater => Some(0),
                std::cmp::Ordering::Less => Some(1),
                std::cmp::Ordering::Equal => None,
            };
        }

        None
    }

    /// Returns `true` when the game has ended.
    ///
    /// The game ends when at least one player is no longer alive (see
    /// [`Player::is_alive`]), or when the action deck has been exhausted
    /// (see [`signal_deck_exhausted`]).
    ///
    /// [`Player::is_alive`]: Player::is_alive
    /// [`signal_deck_exhausted`]: RulesEngine::signal_deck_exhausted
    pub fn is_game_over(&self) -> bool {
        !self.players[0].is_alive() || !self.players[1].is_alive() || self.deck_exhausted
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    fn require_phase(&self, expected: &TurnPhase) -> Result<(), RulesError> {
        if self.phase != *expected {
            return Err(RulesError::WrongPhase {
                expected: expected.clone(),
                actual: self.phase.clone(),
            });
        }
        Ok(())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_engine() -> RulesEngine {
        RulesEngine::new(Player::new("Alice", 3), Player::new("Bob", 3))
    }

    fn ace() -> Card {
        Card::new("Ace", 14)
    }
    fn two() -> Card {
        Card::new("Two", 2)
    }
    fn king() -> Card {
        Card::new("King", 13)
    }

    // ── Player ────────────────────────────────────────────────────────────────

    #[test]
    fn player_new_has_empty_hand_and_correct_life() {
        let p = Player::new("Alice", 5);
        assert_eq!(p.life, 5);
        assert!(p.hand.is_empty());
        assert_eq!(p.name, "Alice");
    }

    #[test]
    fn player_new_starts_with_default_civilians_and_happiness() {
        let p = Player::new("Alice", 5);
        assert_eq!(p.civilians, Player::STARTING_CIVILIANS);
        assert_eq!(p.happiness, Player::STARTING_HAPPINESS);
    }

    #[test]
    fn player_is_alive_when_life_above_zero() {
        let p = Player::new("Alice", 1);
        assert!(p.is_alive());
    }

    #[test]
    fn player_is_not_alive_when_life_is_zero() {
        let p = Player::new("Alice", 0);
        assert!(!p.is_alive());
    }

    #[test]
    fn player_is_not_alive_when_civilians_is_zero() {
        let mut p = Player::new("Alice", 5);
        p.civilians = 0;
        assert!(
            !p.is_alive(),
            "player with no civilians should not be alive"
        );
    }

    #[test]
    fn player_has_civilians_returns_true_when_civilians_above_zero() {
        let p = Player::new("Alice", 3);
        assert!(p.has_civilians());
    }

    #[test]
    fn player_has_civilians_returns_false_when_civilians_is_zero() {
        let mut p = Player::new("Alice", 3);
        p.civilians = 0;
        assert!(!p.has_civilians());
    }

    #[test]
    fn player_adjust_happiness_increases_score() {
        let mut p = Player::new("Alice", 3);
        p.adjust_happiness(10);
        assert_eq!(p.happiness, Player::STARTING_HAPPINESS + 10);
    }

    #[test]
    fn player_adjust_happiness_decreases_score() {
        let mut p = Player::new("Alice", 3);
        p.adjust_happiness(-5);
        assert_eq!(p.happiness, Player::STARTING_HAPPINESS - 5);
    }

    #[test]
    fn player_adjust_happiness_clamps_to_zero() {
        let mut p = Player::new("Alice", 3);
        p.adjust_happiness(-200);
        assert_eq!(p.happiness, 0);
    }

    #[test]
    fn player_adjust_happiness_clamps_to_one_hundred() {
        let mut p = Player::new("Alice", 3);
        p.adjust_happiness(200);
        assert_eq!(p.happiness, 100);
    }

    #[test]
    fn player_receive_card_adds_to_hand() {
        let mut p = Player::new("Alice", 3);
        p.receive_card(ace());
        assert_eq!(p.hand.len(), 1);
    }

    #[test]
    fn player_play_card_removes_card_and_returns_it() {
        let mut p = Player::new("Alice", 3);
        p.receive_card(ace());
        let played = p.play_card(0);
        assert!(played.is_some());
        assert_eq!(played.unwrap().name, "Ace");
        assert!(p.hand.is_empty());
    }

    #[test]
    fn player_play_card_invalid_index_returns_none() {
        let mut p = Player::new("Alice", 3);
        assert!(p.play_card(0).is_none());
    }

    // ── RulesEngine initialisation ────────────────────────────────────────────

    #[test]
    fn engine_starts_at_round_one_draw_phase_player_zero() {
        let engine = make_engine();
        assert_eq!(engine.round, 1);
        assert_eq!(engine.active_player, 0);
        assert_eq!(engine.phase, TurnPhase::Draw);
    }

    #[test]
    fn engine_inactive_player_is_opposite_of_active() {
        let engine = make_engine();
        assert_eq!(engine.inactive_player(), 1);
    }

    // ── draw_card ─────────────────────────────────────────────────────────────

    #[test]
    fn draw_card_advances_phase_to_play() {
        let mut engine = make_engine();
        engine.draw_card(ace()).expect("draw should succeed");
        assert_eq!(engine.phase, TurnPhase::Play);
    }

    #[test]
    fn draw_card_adds_card_to_active_player_hand() {
        let mut engine = make_engine();
        engine.draw_card(ace()).expect("draw should succeed");
        assert_eq!(engine.players[0].hand.len(), 1);
        assert_eq!(engine.players[1].hand.len(), 0);
    }

    #[test]
    fn draw_card_fails_in_wrong_phase() {
        let mut engine = make_engine();
        engine.draw_card(ace()).expect("first draw ok");
        let err = engine.draw_card(king()).unwrap_err();
        assert_eq!(
            err,
            RulesError::WrongPhase {
                expected: TurnPhase::Draw,
                actual: TurnPhase::Play,
            }
        );
    }

    // ── play_card ─────────────────────────────────────────────────────────────

    #[test]
    fn play_card_advances_phase_to_battle() {
        let mut engine = make_engine();
        engine.draw_card(ace()).unwrap();
        engine.play_card(0).expect("play should succeed");
        assert_eq!(engine.phase, TurnPhase::Battle);
    }

    #[test]
    fn play_card_removes_card_from_hand() {
        let mut engine = make_engine();
        engine.draw_card(ace()).unwrap();
        let card = engine.play_card(0).expect("play should succeed");
        assert_eq!(card.name, "Ace");
        assert!(engine.players[0].hand.is_empty());
    }

    #[test]
    fn play_card_fails_with_invalid_index() {
        let mut engine = make_engine();
        engine.draw_card(ace()).unwrap();
        let err = engine.play_card(5).unwrap_err();
        assert_eq!(err, RulesError::InvalidCardIndex(5));
    }

    #[test]
    fn play_card_fails_in_wrong_phase() {
        let mut engine = make_engine();
        let err = engine.play_card(0).unwrap_err();
        assert_eq!(
            err,
            RulesError::WrongPhase {
                expected: TurnPhase::Play,
                actual: TurnPhase::Draw,
            }
        );
    }

    // ── resolve_battle ────────────────────────────────────────────────────────

    fn advance_to_battle(engine: &mut RulesEngine, attacker_card: Card) {
        engine.draw_card(attacker_card).unwrap();
        engine.play_card(0).unwrap();
    }

    #[test]
    fn battle_attacker_wins_reduces_defender_life() {
        let mut engine = make_engine();
        advance_to_battle(&mut engine, ace());
        let outcome = engine.resolve_battle(&ace(), &two()).expect("resolve ok");
        assert_eq!(outcome, BattleOutcome::AttackerWins);
        // Active player 0 attacked; player 1 (defender) loses 1 life.
        assert_eq!(engine.players[0].life, 3);
        assert_eq!(engine.players[1].life, 2);
    }

    #[test]
    fn battle_defender_wins_reduces_attacker_life() {
        let mut engine = make_engine();
        advance_to_battle(&mut engine, two());
        let outcome = engine.resolve_battle(&two(), &ace()).expect("resolve ok");
        assert_eq!(outcome, BattleOutcome::DefenderWins);
        // Player 0 attacked but lost; player 0 loses 1 life.
        assert_eq!(engine.players[0].life, 2);
        assert_eq!(engine.players[1].life, 3);
    }

    #[test]
    fn battle_draw_no_life_lost() {
        let mut engine = make_engine();
        advance_to_battle(&mut engine, king());
        let outcome = engine.resolve_battle(&king(), &king()).expect("resolve ok");
        assert_eq!(outcome, BattleOutcome::Draw);
        assert_eq!(engine.players[0].life, 3);
        assert_eq!(engine.players[1].life, 3);
    }

    #[test]
    fn battle_advances_phase_to_end() {
        let mut engine = make_engine();
        advance_to_battle(&mut engine, ace());
        engine.resolve_battle(&ace(), &two()).unwrap();
        assert_eq!(engine.phase, TurnPhase::End);
    }

    #[test]
    fn battle_fails_in_wrong_phase() {
        let mut engine = make_engine();
        let err = engine.resolve_battle(&ace(), &two()).unwrap_err();
        assert_eq!(
            err,
            RulesError::WrongPhase {
                expected: TurnPhase::Battle,
                actual: TurnPhase::Draw,
            }
        );
    }

    // ── end_turn ──────────────────────────────────────────────────────────────

    fn advance_to_end(engine: &mut RulesEngine) {
        engine.draw_card(ace()).unwrap();
        engine.play_card(0).unwrap();
        engine.resolve_battle(&ace(), &two()).unwrap();
    }

    #[test]
    fn end_turn_passes_to_other_player() {
        let mut engine = make_engine();
        advance_to_end(&mut engine);
        engine.end_turn().expect("end turn ok");
        assert_eq!(engine.active_player, 1);
    }

    #[test]
    fn end_turn_increments_round() {
        let mut engine = make_engine();
        advance_to_end(&mut engine);
        engine.end_turn().unwrap();
        assert_eq!(engine.round, 2);
    }

    #[test]
    fn end_turn_resets_phase_to_draw() {
        let mut engine = make_engine();
        advance_to_end(&mut engine);
        engine.end_turn().unwrap();
        assert_eq!(engine.phase, TurnPhase::Draw);
    }

    #[test]
    fn end_turn_fails_in_wrong_phase() {
        let mut engine = make_engine();
        let err = engine.end_turn().unwrap_err();
        assert_eq!(
            err,
            RulesError::WrongPhase {
                expected: TurnPhase::End,
                actual: TurnPhase::Draw,
            }
        );
    }

    // ── winner / is_game_over ─────────────────────────────────────────────────

    #[test]
    fn no_winner_when_both_players_alive() {
        let engine = make_engine();
        assert!(engine.winner().is_none());
        assert!(!engine.is_game_over());
    }

    #[test]
    fn player_one_wins_when_player_two_life_hits_zero() {
        let mut engine = RulesEngine::new(Player::new("Alice", 2), Player::new("Bob", 1));
        // Bob starts with 1 life; one attacker win is enough.
        advance_to_end(&mut engine); // Alice attacks and wins (ace vs two).
        assert_eq!(engine.winner(), Some(0));
        assert!(engine.is_game_over());
    }

    #[test]
    fn player_two_wins_when_player_one_life_hits_zero() {
        let mut engine = RulesEngine::new(Player::new("Alice", 1), Player::new("Bob", 2));
        // Alice attacks with a low card and loses.
        engine.draw_card(two()).unwrap();
        engine.play_card(0).unwrap();
        engine.resolve_battle(&two(), &ace()).unwrap();
        assert_eq!(engine.winner(), Some(1));
        assert!(engine.is_game_over());
    }

    // ── deck exhaustion ───────────────────────────────────────────────────────

    #[test]
    fn deck_not_exhausted_by_default() {
        let engine = make_engine();
        assert!(!engine.deck_exhausted);
        assert!(!engine.is_game_over());
    }

    #[test]
    fn signal_deck_exhausted_ends_game() {
        let mut engine = make_engine();
        engine.signal_deck_exhausted();
        assert!(engine.is_game_over());
    }

    #[test]
    fn deck_exhausted_player_with_higher_happiness_wins() {
        let mut engine = make_engine();
        engine.players[0].adjust_happiness(10); // Alice: 60, Bob: 50
        engine.signal_deck_exhausted();
        assert_eq!(engine.winner(), Some(0));
    }

    #[test]
    fn deck_exhausted_player_two_higher_happiness_wins() {
        let mut engine = make_engine();
        engine.players[1].adjust_happiness(20); // Bob: 70, Alice: 50
        engine.signal_deck_exhausted();
        assert_eq!(engine.winner(), Some(1));
    }

    #[test]
    fn deck_exhausted_equal_happiness_is_a_draw() {
        let mut engine = make_engine();
        // Both players start at STARTING_HAPPINESS (50); no adjustments.
        engine.signal_deck_exhausted();
        assert!(engine.winner().is_none());
        // is_game_over is still true even on a draw.
        assert!(engine.is_game_over());
    }

    #[test]
    fn no_civilians_takes_priority_over_deck_exhaustion() {
        let mut engine = make_engine();
        // Player 0 loses their civilians.
        engine.players[0].civilians = 0;
        engine.signal_deck_exhausted();
        // Player 1 wins via civilian loss, not happiness.
        assert_eq!(engine.winner(), Some(1));
    }

    // ── full round trip ───────────────────────────────────────────────────────

    #[test]
    fn full_two_round_game() {
        // Alice: 2 life, Bob: 2 life.
        // Round 1: Alice attacks with Ace (14) vs King (13) → Alice wins, Bob loses 1 life (1 remaining).
        // Round 2: Bob attacks with Ace (14) vs Two (2) → Bob wins, Alice loses 1 life (1 remaining).
        let mut engine = RulesEngine::new(Player::new("Alice", 2), Player::new("Bob", 2));

        // Round 1 – Alice's turn.
        engine.draw_card(ace()).unwrap();
        let attacker = engine.play_card(0).unwrap();
        let outcome = engine.resolve_battle(&attacker, &king()).unwrap();
        assert_eq!(outcome, BattleOutcome::AttackerWins);
        assert_eq!(engine.players[1].life, 1);
        engine.end_turn().unwrap();

        // Round 2 – Bob's turn (active_player == 1).
        assert_eq!(engine.active_player, 1);
        engine.draw_card(ace()).unwrap();
        let attacker = engine.play_card(0).unwrap();
        let outcome = engine.resolve_battle(&attacker, &two()).unwrap();
        assert_eq!(outcome, BattleOutcome::AttackerWins);
        assert_eq!(engine.players[0].life, 1);
        engine.end_turn().unwrap();

        assert_eq!(engine.round, 3);
        assert!(engine.winner().is_none());
    }
}
