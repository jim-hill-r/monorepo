# 0005. Main NFL tiebreaker steps, coin flip for the rest

- Status: Accepted
- Date: 2026-11-25

## Context

Seeding decides who makes the playoffs and who gets the bye, and ties are
common in simulated seasons. The full NFL procedure has a dozen steps,
including points-based ones that the simulator cannot model because it does
not simulate scores.

## Decision

Implement these steps fully, for both two-team and three-or-more-team ties,
including the rule that division ties are broken before wild-card ties and
restarting the procedure when a multi-team tie is reduced.

Division ties:

1. Head-to-head record
2. Division record
3. Record in common games
4. Conference record
5. Strength of victory
6. Strength of schedule

Wild-card ties:

1. Head-to-head record (only on a sweep when three or more teams are tied)
2. Conference record
3. Record in common games (minimum of four)
4. Strength of victory
5. Strength of schedule

Any tie still standing after these steps is broken by a coin flip (random
draw in the simulation).

## Consequences

- The steps that rely on points (combined ranking, net points, net
  touchdowns) are skipped. They decide very few real seedings.
- Tests check the implementation against real final standings and seeds from
  past seasons.
