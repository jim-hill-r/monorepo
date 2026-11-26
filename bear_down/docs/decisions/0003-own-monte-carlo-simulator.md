# 0003. Our own Monte Carlo season simulator in Rust

- Status: Accepted
- Date: 2026-11-25

## Context

Public playoff-odds sites publish each team's chances, but not how each
remaining game changes a particular team's chances. Getting "who to root for"
requires running the season ourselves.

## Decision

- Write a Monte Carlo simulator in `bear_down_core`:
  1. Start from the actual results so far.
  2. Play every remaining regular-season game using the win probability from
     [0004](0004-elo-win-probabilities.md). Simulated games never tie.
  3. Build standings and seeds with [0005](0005-tiebreakers.md).
  4. Play the playoff bracket game by game, with home field for the higher
     seed and a neutral-site Super Bowl.
  5. Repeat about 100,000 times.
- For each game this week, split the simulations by who won it and compare the
  Bears' outcomes in the two groups. That gives the swing in 0002 without a
  separate run per game.
- The random seed is fixed per season and week, so the same input always
  produces the same page.
- Elo ratings are not updated inside a simulated season (no "hot hand").

## Consequences

- Run time and memory stay small enough for a CI job.
- Small swings are within Monte Carlo noise. The analysis reports a standard
  error so 0002 can label those games "doesn't matter".
- Correctness depends heavily on the tiebreakers and seeding, which need
  tests against real past seasons.
