---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Rulebook mechanics
order: 20
added: 2026-11-28
---

Deal profession cards at setup, as [rulebook.md](../docs/rulebook.md) says: each player is dealt three and keeps one, and the other two leave the game. Today Engineer and Farmer are plain value cards in the action deck ([official_rules.md](../docs/official_rules.md#not-built-yet)). The rulebook lists no profession effects yet, so propose what a kept profession does in official_rules.md in a PR labelled `needs-human` for Jim before building it in `src/rules.rs` and the start of a game in `src/game.rs` (the CPU picks with `src/ai.rs`). Show the kept profession on screen, and add CHAOS-002 evals in `evals.toml`.
