# bear_down

A small static website that tells a Chicago Bears fan which NFL games to root
for each week to give the Bears the best chance of winning the Super Bowl.

Every game on the week's slate is ranked by how much its result moves the
Bears' Super Bowl odds, with the team to root for and the size of the swing
(for example "+1.8% Super Bowl odds"). An opt-in toggle switches the rankings
to a different goal: getting all four NFC North teams into the divisional
round.

There is no backend. A Rust simulator runs at build time, the site is rebuilt
and redeployed every week, and Cloudflare serves the result as static files.

The name comes from "Bear Down, Chicago Bears", the team's fight song.

## Status

Early development. `core` parses the nflverse schedule and results
(`bear_down_core::Schedule`) and builds Elo ratings and single-game win
probabilities from them (`bear_down_core::Ratings`). It also computes
standings, division winners and playoff seeds with the NFL tiebreakers from
decision 0005 (`bear_down_core::Standings`), checked against every real
playoff field since 2002. `bear_down_core::simulate` plays out the rest of
the season and the playoffs (100,000 times for a weekly build, about half a
second in a release build) and gives every team's playoff and Super Bowl
odds, plus each of this week's games' swing on the Bears' playoff and Super
Bowl odds and on the NFC North divisional-round goal. The `bear_down` CLI downloads
the nflverse schedule, runs the simulation and writes `rooting.json`
([format](docs/rooting-json.md)).
The decisions are recorded in [docs/decisions](docs/decisions/README.md) and
the remaining work to build v1 is listed in [ISSUES.md](ISSUES.md).

## Layout

| Crate | Package | Does |
| --- | --- | --- |
| `core/` | `bear_down_core` | Data parsing, Elo, standings, simulation and rooting analysis. No I/O. |
| `cli/` | `bear_down_cli` (binary `bear_down`) | Downloads the data, runs the simulation, writes `rooting.json`. |
| `web/` | `bear_down_web` | The Dioxus page that renders `rooting.json`. |
