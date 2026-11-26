# Priority Issues

- TODO (agent-generated): Make the status text in `update_pvp_ui` (`src/game.rs`) testable. Extract the hand listing and the instruction message into pure functions, such as `hand_text(&RulesEngine, hide_ai_hand: bool) -> String` and `instructions_text(&RulesEngine, deck_remaining, attacker: Option<&Card>, ai_present: bool) -> String`, and add unit tests for each game-over message (deck exhausted with a winner, deck exhausted with a draw, life reaching 0, civilians reaching 0), the hidden AI hand, and the "CPU is taking its turn" state.
- TODO (agent-generated): Remove duplication in the input handlers in `src/game.rs`. `handle_play_phase` and `handle_battle_phase` each build the same `[(KeyCode::Digit1, 0) ... (KeyCode::Digit8, 7)]` table; replace both with one `pressed_hand_index(&ButtonInput<KeyCode>, hand_len) -> Option<usize>` helper with unit tests. In the same change, merge the duplicated empty-hand defence (`Card::new(EMPTY_HAND_CARD, 0)` in both `handle_battle_phase` and `ai_handle_battle_phase`) into one function, and delete the `GameState` resource and `log_game_state`, whose `turn` field is never updated so the log always says "Turn: 0".

# Backlog

# Priority Projects

# On Hold Projects
