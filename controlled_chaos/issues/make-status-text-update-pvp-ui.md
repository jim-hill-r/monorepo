---
kind: todo
priority: priority
tags: [agent-generated]
added: 2026-11-26
---

Make the status text in `update_pvp_ui` (`src/game.rs`) testable. Extract the hand listing and the instruction message into pure functions, such as `hand_text(&RulesEngine, hide_ai_hand: bool) -> String` and `instructions_text(&RulesEngine, deck_remaining, attacker: Option<&Card>, ai_present: bool) -> String`, and add unit tests for each game-over message (deck exhausted with a winner, deck exhausted with a draw, life reaching 0, civilians reaching 0), the hidden AI hand, and the "CPU is taking its turn" state.
