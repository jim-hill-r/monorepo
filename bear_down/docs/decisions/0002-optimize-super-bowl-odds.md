# 0002. Rank every game by its swing on Bears Super Bowl odds

- Status: Accepted
- Date: 2026-11-25

## Context

"Who should a Bears fan root for?" needs one number to optimize. The goal is
the Super Bowl, but early in a season most games barely move Super Bowl odds,
so that number alone can make every game look irrelevant.

## Decision

- The headline metric is the Bears' probability of winning the Super Bowl.
- Every game in the current week is listed, sorted by the size of its swing:
  the Bears' Super Bowl odds if one team wins minus the odds if the other
  team wins. Each row shows the team to root for and the swing, for example
  "+1.8% Super Bowl odds".
- The Bears' playoff odds and the swing on playoff odds are shown alongside,
  since they are larger and easier to feel.
- The Bears' own game is listed like any other (the answer is always "Bears").
- Swings smaller than the simulation's noise are shown as "doesn't matter"
  rather than as a pick.
- There is no draft-position mode once the Bears are eliminated.

## Consequences

- The simulator must estimate both Super Bowl and playoff odds conditioned on
  each game's result (see [0003](0003-own-monte-carlo-simulator.md)).
- If the Bears are eliminated, the page says so instead of ranking games.
