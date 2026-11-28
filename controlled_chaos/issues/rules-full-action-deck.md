---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Rulebook mechanics
order: 40
added: 2026-11-28
---

Grow the action deck from 18 cards (`DEFAULT_ACTION_DECK` in `src/card_library.rs`) toward the rulebook's 52: 13 each of Technology, Government, Environment and Economy, every card with an action, a passive and power-ups when cards are placed above, below, left or right of it ([rulebook.md](../docs/rulebook.md), "Actions" and "Action Cards", still blank). This needs Jim's card designs first: propose a first set of cards and one power-up rule in official_rules.md in a PR labelled `needs-human`, and build them once he agrees. Keep `MAX_COPIES_PER_CARD` and the seeded shuffle, and update the deck table in [official_rules.md](../docs/official_rules.md#action-deck-18-cards).
