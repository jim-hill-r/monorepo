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

Planning. The initial decisions are recorded in
[docs/decisions](docs/decisions/README.md) and the work to build v1 is listed
in [ISSUES.md](ISSUES.md).
