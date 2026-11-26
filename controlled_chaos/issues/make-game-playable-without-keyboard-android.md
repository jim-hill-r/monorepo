---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-26
---

Make the game playable without a keyboard, so the Android APK and phone browsers work. Cards in hand and the phase change are keyboard-only today (`KeyCode::Digit1`..`Digit8` and `Space` in `src/game.rs`): add tap and click targets for each card in hand and a "Next phase" button that feed the same actions, with unit tests for the shared action function.
