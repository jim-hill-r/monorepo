# bear_down

[![Eval coverage](https://raw.githubusercontent.com/jimhillr/monorepo/evals-results/badges/bear_down.svg)](https://github.com/jimhillr/monorepo/blob/evals-results/verdicts.json)

A small static website that tells a Chicago Bears fan which NFL games to root
for each week to give the Bears the best chance of winning the Super Bowl.
A fan of any other team can pick theirs and get the same guide for it
([decision 0010](docs/decisions/0010-any-team.md)).

Every game on the week's slate is ranked by how much its result moves the
Bears' Super Bowl odds, with the team to root for and the size of the swing
(for example "+1.8% Super Bowl odds"). An opt-in toggle switches the rankings
to a different goal: getting all four NFC North teams into the divisional
round.

There is no backend. A Rust simulator runs at build time, the site is rebuilt
and redeployed every week, and Cloudflare serves the result as static files.

The name comes from "Bear Down, Chicago Bears", the team's fight song.

How to use the site is in the [user guide](docs/user/README.md).

## Status

Early development. `core` parses the nflverse schedule and results
(`bear_down_core::Schedule`) and builds Elo ratings and single-game win
probabilities from them (`bear_down_core::Ratings`), with parameters
tuned against past seasons ([docs/elo-tuning.md](docs/elo-tuning.md)). It also computes
standings, division winners and playoff seeds with the NFL tiebreakers from
decision 0005 (`bear_down_core::Standings`), checked against every real
playoff field since 2002. `bear_down_core::simulate` plays out the rest of
the season and the playoffs (100,000 times for a weekly build, about half a
second in a release build) and gives every team's playoff and Super Bowl
odds, plus each of this week's games' swing on the Bears' playoff and Super
Bowl odds and on the NFC North divisional-round goal. The `bear_down` CLI downloads
the nflverse schedule, runs the simulation and writes `rooting.json`
([format](docs/rooting-json.md)), and with `--teams-dir` every other
team's guide as `teams/<team>.json` for the page's team picker.
`bear_down --week <n> [--season <year>]` rebuilds a past week's guide from
the results before it, for the [archive](docs/plans/2026-11-archive.md).
The decisions are recorded in [docs/decisions](docs/decisions/README.md) and
the remaining work to build v1 is listed in [issues/](issues/).

## Layout

| Crate | Package | Does |
| --- | --- | --- |
| `core/` | `bear_down_core` | Data parsing, Elo, standings, simulation and rooting analysis. No I/O. |
| `cli/` | `bear_down_cli` (binary `bear_down`) | Downloads the data, runs the simulation, writes `rooting.json`. |
| `web/` | `bear_down_web` | The Dioxus page that renders `rooting.json`. |
| `design/` | (not a crate) | The "Chicago Bears" theme for the [design system](../design_system/README.md): `theme.toml` is the source, `theme.css` is generated from it. |

## Theme

The web page uses the shared design system. Its own "Chicago Bears" theme in
[design/theme.toml](design/theme.toml) extends the standard colorful theme
with Bears navy and orange, in light, dark and accessible mode, and the
page's appearance picker also offers the five standard themes. After
changing `theme.toml`, run `mise run render` in `design_system` to
regenerate `theme.css`; `design_system`'s CI fails until you do, and also
fails if any colour pair misses its WCAG contrast minimum.
