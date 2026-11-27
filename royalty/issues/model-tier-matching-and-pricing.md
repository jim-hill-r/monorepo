---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Royalty / Phase 1: MVP scanner and report"
order: 20
added: 2026-11-27
---

In `royalty_model`, add the pure calculation for one producer family and one consumer: the built-in free non-production tier, first-match tier selection on the `when` conditions, and pricing (`included`, `max_units`, `cents` per `per`, `basis_points`) in integer millionths of a cent, rounded down to a whole cent once per family per period. Steps 3 to 5 of [How the amount is worked out](../docs/plans/2026-11-royalty-plan.md#how-the-amount-is-worked-out). Tests cover each condition, each unit, rounding, and that no tier can charge a non-production environment.
