# Test fixtures

`games.csv` holds 13 rows cut unchanged from nflverse
[`games.csv`](https://github.com/nflverse/nfldata/blob/master/data/games.csv),
downloaded 2026-11-26, with the original header and all 46 columns. The rows
were picked to cover the cases the parser handles:

- regular-season home and away wins, and a tie (`2026_05_NYG_WAS`)
- neutral sites: an international game (`2026_01_SF_LA`) and the Super Bowl
- every playoff round but the wild card (`DIV`, `CON`, `SB`), with the
  2025 playoffs dated in 2026
- unplayed games with empty scores (`2026_12_*`)
- a quoted empty column (`2022_04_MIN_NO`) and the oldest season (1999)
- week 11 listed after week 12, to test sorting

To refresh, copy the same game ids out of a new download.

## seasons.csv

`seasons.csv` holds every 2002 to 2025 game, regular season and playoffs,
from the same download. It keeps only the 13 columns the parser reads, in
their original order, to stay small. The standings tests replay each season
and check the seeds against the real playoff games. 2002 is the first season
with today's divisions.

To refresh or add a season, filter a new download to those seasons and
columns.
