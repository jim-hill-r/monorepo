---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Rulebook mechanics
order: 30
added: 2026-11-28
---

Make civilians cards rather than a counter, as [rulebook.md](../docs/rulebook.md) ("Civilians") says: each player starts with two, one face up and one face down; each has a population and an upkeep; a player who can't meet upkeep at the start of a turn acts on a civilian card; and low happiness has consequences. Today `Player::civilians` in `src/rules.rs` starts at 2 and nothing changes it, so the "no civilians" loss can't happen in play ([official_rules.md](../docs/official_rules.md#civilians)). Propose the upkeep and the effects that gain or lose civilians in official_rules.md in a PR labelled `needs-human` for Jim, then build them with CHAOS-002 evals.
