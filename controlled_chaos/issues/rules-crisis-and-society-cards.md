---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Rulebook mechanics
order: 10
added: 2026-11-28
---

Play the crisis and society cards from [rulebook.md](../docs/rulebook.md) ("Society/Crisis" and "Play"). Today Asteroid and Flood (Crisis) and Space Station (Society) are plain value cards in the action deck ([official_rules.md](../docs/official_rules.md#not-built-yet)). Setup should slide one random crisis card about halfway into the action deck and put one at the bottom, and remove one society card at random. When a crisis card is drawn, every player loses unless the matching society card averts it (Asteroid is stopped by Space Station). The rulebook doesn't say how a society card is played or what happens with no match, so first write the proposed rules into official_rules.md in a PR labelled `needs-human` for Jim, and build them in the RulesEngine (`src/rules.rs`) once he agrees. Add a CHAOS-002 eval for each new rule in `evals.toml`.
