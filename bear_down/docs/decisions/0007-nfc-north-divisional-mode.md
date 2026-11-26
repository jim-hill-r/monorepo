# 0007. Opt-in NFC North divisional-round mode

- Status: Accepted
- Date: 2026-11-25

## Context

Jim wants an optional view that shows the best rooting pattern for all four
NFC North teams (Bears, Lions, Packers, Vikings) to reach the divisional
round. That requires all four to make the playoffs, one of them as the NFC
#1 seed with the bye, and the other three to win their wild-card games, so no
two of them can meet in the wild-card round.

## Decision

- The goal is exactly as stated: the probability that **all four** NFC North
  teams reach the divisional round, not "as many as possible".
- The page has a toggle, off by default, that switches the rankings from the
  Bears' Super Bowl odds to this probability.
- The same simulation run produces both rankings, and both go into
  `rooting.json`, so the toggle is instant and needs no extra build.
- The probability is shown even when it is tiny.

## Consequences

- The outcome is rare, so its swings are often within Monte Carlo noise. If
  that makes the rankings useless, a later decision can raise the simulation
  count for this mode or change the goal.
- In this mode a Bears fan will sometimes be told to root against the Bears.
