# Elo tuning

How the Elo parameters in `bear_down_core::EloParams` were tuned against
past seasons ([decision 0004](decisions/0004-elo-win-probabilities.md)).
Done 2026-11-27 on `core/fixtures/seasons.csv` (every 2002 to 2025 game).

## Method

`bear_down_core::elo::brier_score` replays every played game oldest first
and scores the home team's win probability just before each game in the
chosen seasons: the mean of `(p - result)^2`, with a tie counted as 0.5.
Lower is better, and always guessing 0.5 scores 0.25. 2002 to 2004 only
warm the ratings up.

Each grid was tuned on one block of seasons and checked on the next, so a
value that only fits the past doesn't win:

| Tuned on | Best there (K, home field, regression) | Checked on | Tuned | Starting values |
| --- | --- | --- | --- | --- |
| 2005 to 2014 | 22, 55, 0.5 | 2015 to 2018 | 0.21975 | 0.22014 |
| 2010 to 2018 | 20, 55, 0.5 | 2019 to 2021 | 0.22479 | 0.22254 |
| 2014 to 2021 | 18, 35, 1/3 | 2022 to 2025 | 0.22168 | 0.22178 |

Tuning K and the season regression on old seasons made the next block
worse or barely better, so they stay at K = 20 and one third. The one
change that holds up is home field.

## Home field

The home edge has shrunk. Before 2019 about 50 Elo points scores best;
since 2019, 20 to 35 does (2020, with almost no fans, did best at zero).

| Home field | 2005 to 2018 | 2019 to 2025 | 2022 to 2025 |
| --- | --- | --- | --- |
| 20 | 0.22009 | 0.22100 | 0.22184 |
| 25 | 0.21949 | 0.22098 | 0.22162 |
| 30 | 0.21898 | 0.22106 | 0.22149 |
| 35 | 0.21856 | 0.22124 | 0.22145 |
| 48 (starting value) | 0.21791 | 0.22210 | 0.22178 |

The default is now **30**, between the best values for 2019 to 2025 and
2022 to 2025. It scores 0.2211 on 2019 to 2025, against 0.2221 with 48.

## Margin multiplier

Turning the margin multiplier off (`EloParams::margin = false`) scores
0.2266 on 2019 to 2025, much worse, so it stays on.

## Against betting lines

The betting lines in the same file (`Φ(spread_line / 13.45)`) score 0.2105
on 2019 to 2025, well ahead of any Elo setting. That is why every game with
a line uses it (decision 0004's amendment), and Elo covers only later games
and the playoffs.

## Checking it again

`tuned_defaults_beat_the_starting_values_on_recent_seasons` in
`core/src/elo.rs` fails if the defaults stop beating home field 48 on 2019
to 2025, or the margin multiplier stops helping. After adding a season to
`seasons.csv`, rerun the grids with `brier_score` and update this file.
