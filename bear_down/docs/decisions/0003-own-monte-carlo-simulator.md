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
- For each game this week, play every simulated season twice from the same
  seed, once with each result (common random numbers), and compare the
  Bears' outcomes in each pair. The other games come out the same in both,
  so the difference comes from this game alone.
  - Amended 2026-11-26: v1 first split one set of simulations by who won
    each game. The other games' randomness swamped small swings, so with
    100,000 runs most games showed "doesn't matter", which isn't true this
    late in a season (Jim). Pairing costs two extra runs per game (about 40
    seconds for a week in a release build) and cuts the noise to what the
    game itself causes.
- The random seed is fixed per season and week, so the same input always
  produces the same page.
- Elo ratings are not updated inside a simulated season (no "hot hand").

## Consequences

- Run time and memory stay small enough for a CI job.
- Small swings are within Monte Carlo noise. The analysis reports the
  standard error of the paired difference so 0002 can label only games that
  truly barely move the odds "doesn't matter".
- Correctness depends heavily on the tiebreakers and seeding, which need
  tests against real past seasons.
