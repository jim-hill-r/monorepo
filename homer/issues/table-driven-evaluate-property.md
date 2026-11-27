---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

`evaluate_property` in `src/main.rs` repeats the same five `match` blocks (set, met, not met, unknown) and counts the set criteria again in a separate `criteria_checked` array, so adding a criterion in one place but not the other silently breaks the score (`criteria_checked - unmet_criteria.len()` could even underflow). Replace them with one small `Criterion` helper (label, preference, value, comparison, formatter) that returns met/unmet/not set, and derive both `criteria_checked` and `unmet_criteria` from the same list. Keep the existing unit and transcript tests unchanged as the check that output didn't move.
