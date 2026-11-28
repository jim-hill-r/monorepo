---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Fan feedback
order: 80
added: 2026-11-27
---

Step 6 of [the fan feedback plan](../plans/2026-11-fan-feedback.md): once the decision point is reached (200 votes per arm, or the end of the 2026 regular season, whichever comes first), write the retro in that plan from the weekly summaries on the `feedback-data` branch (`bear_down/<season>-week-<nn>.md`). Say which arm's "doesn't matter" line fans agreed with more, what the notes said, and what changes. If arm B (0.02 points) wins, open a requirement-only PR changing BEAR-005 for Jim's approval; if arm A wins, retire the A/B split in `bear_down/web/src/feedback.rs` so every fan sees the published line. Until the decision point, leave this open: the weekly `telemetry-feedback-export.yml` run keeps the data coming.
