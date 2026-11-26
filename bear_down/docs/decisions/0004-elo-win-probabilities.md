# 0004. Elo ratings for single-game win probabilities

- Status: Accepted
- Date: 2026-11-25

## Context

The simulator needs a probability for every remaining game. Options were Elo
ratings computed from results, betting point spreads, or a mix. Spreads are
only reliable for the coming week and good sources usually need a paid API.

## Decision

- Use Elo only for v1, computed from nflverse results
  ([0006](0006-nflverse-data.md)).
- Ratings are rebuilt from historical seasons on every build, so the build
  needs nothing stored from earlier runs.
- Between seasons, each rating moves one third of the way back to the
  average (1505).
- Starting parameters, to be tuned against past seasons: K = 20, home-field
  advantage of about 48 Elo points (none at neutral sites), and a
  margin-of-victory multiplier.
- Win probability is `1 / (1 + 10^(-diff / 400))`, where `diff` includes
  home-field advantage.

## Consequences

- No API keys and no paid data.
- Elo ignores injuries and quarterback changes, so it is weaker than the
  betting market for the current week.
- Amended 2026-11-26 (Jim: "we are leaning too heavily on Elo"): every
  remaining game with a betting line in `games.csv` (nflverse publishes
  them about two weeks ahead) uses the line instead, as
  `Φ(spread_line / 13.45)`. Elo covers later games and the playoffs.
