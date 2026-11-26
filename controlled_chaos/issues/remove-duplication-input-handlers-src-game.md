---
kind: todo
priority: priority
tags: [agent-generated]
added: 2026-11-26
---

Remove duplication in the input handlers in `src/game.rs`. `handle_play_phase` and `handle_battle_phase` each build the same `[(KeyCode::Digit1, 0) ... (KeyCode::Digit8, 7)]` table; replace both with one `pressed_hand_index(&ButtonInput<KeyCode>, hand_len) -> Option<usize>` helper with unit tests. In the same change, merge the duplicated empty-hand defence (`Card::new(EMPTY_HAND_CARD, 0)` in both `handle_battle_phase` and `ai_handle_battle_phase`) into one function, and delete the `GameState` resource and `log_game_state`, whose `turn` field is never updated so the log always says "Turn: 0".
