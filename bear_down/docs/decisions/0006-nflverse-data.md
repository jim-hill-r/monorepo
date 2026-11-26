# 0006. Schedule and results from nflverse games.csv

- Status: Accepted
- Date: 2026-11-25

## Context

The simulator needs the full schedule, final scores and history going back
far enough to build Elo ratings. ESPN's JSON endpoints are fresher but
undocumented and can change without notice.

## Decision

- Use the `games.csv` file published by the nflverse project
  (`nflverse/nfldata` on GitHub). It is free, needs no key, covers every
  season since 1999, and includes home/away, neutral-site flags, scores and
  point spreads.
- The `cli` downloads the file at build time. Tests use small checked-in
  fixtures cut from it.

## Consequences

- nflverse usually updates within hours of games ending, which fits a
  Tuesday build. If the file is late, the manual workflow run covers it.
- The page should show the latest game included so stale data is visible.
- Team abbreviations follow nflverse (for example `CHI`, `GB`, `DET`, `MIN`).
