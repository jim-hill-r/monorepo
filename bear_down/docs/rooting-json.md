# rooting.json

The `bear_down` CLI writes `rooting.json`, the Bears' guide, and the web
page fetches it from the same origin at page load (decision 0001). With
`--teams-dir` it also writes every other team's guide in the same format,
as `teams/<team>.json` such as `teams/KC.json`, which the page fetches
when a fan picks that team (decision 0010). Everything below about "the
Bears" holds for the guide's team. The Rust types are in
`cli/src/rooting.rs`; change this file with them.

Probabilities are shares from 0 to 1. Teams are nflverse abbreviations, for
example `CHI`, `DET`, `GB`, `MIN`. Dates are `YYYY-MM-DD`.

| Field | Type | Meaning |
| --- | --- | --- |
| `generated_at` | string | When the guide was built, RFC 3339 UTC. |
| `season` | number | The season simulated. |
| `week` | number or null | The current week: the first with a game left. Null once the regular season is over. |
| `simulations` | number | How many seasons were simulated. |
| `latest_game` | object or null | The latest game with a final score: `id`, `gameday`, `away`, `home`, `away_score`, `home_score`. Shows how fresh the data is (decision 0006). |
| `team` | string | The team the guide is for: `CHI` in `rooting.json`. Missing in older files, which are the Bears'. |
| `focus.record` | object | `wins`, `losses`, `ties`. (`focus` was `bears` in older files; the page reads both.) |
| `focus.division_lead` | number | Games ahead of the best rival in the team's division, by record alone (no tiebreakers; a tie counts half): positive is alone in first place, 0 is tied for it, negative is behind. |
| `focus.division_rival` | string | That best rival. |
| `focus.odds` | object | `playoffs`, `divisional_round`, `conference_game`, `super_bowl` (reached it), `champion` (won it). |
| `banner` | object or null | The celebration banner, worked out every build from the current results, or null unless the team's most recent game (regular season or playoffs) was a win. `headline` is the score ("Bears win! Chicago 39, Lions 35."); `lines` holds up to two notes, the first that apply in this order: the next playoff round, the NFC North race (clinched, alone in first or tied for it), a clear move in Super Bowl or playoff odds since just before that week (from a second simulation as of then), a winning streak of three or more, and the record. Missing in older files. The rules are in `cli/src/banner.rs`. |
| `nfc_north_divisional` | number | The chance all four NFC North teams reach the divisional round (decision 0007). The page shows it, and the `nfc_north` goal, only for an NFC North team's guide. |
| `games` | array | The current week's games still to be played, in schedule order. |

Each game has `id`, `gameday`, `away`, `home`, `home_win_probability` and one
swing object per goal:

- `champion`: the Bears winning the Super Bowl, the default goal (decision 0002).
- `playoffs`: the Bears making the playoffs.
- `nfc_north`: all four NFC North teams reaching the divisional round.

A swing object has:

- `if_home_wins` and `if_away_wins`: the goal's odds after each result.
- `swing`: `if_home_wins - if_away_wins`; negative means root for the away
  team.
- `std_error`: the standard error of `swing` (null if nothing was
  simulated). The page uses it to apply its A/B arm's "doesn't matter" line
  with the CLI's rule, `bear_down_core::tier_against`
  ([fan feedback plan](plans/2026-11-fan-feedback.md)).
- `root_for`: the team to root for, or null for "doesn't matter". For the
  Super Bowl and playoff goals a pick needs a clear direction (more than two
  standard errors) and a swing that isn't confidently under 0.05 points
  (BEAR-005).
- `tier`: `matters` (at least 0.05 points, with confidence), `small` (too
  close to the line to call), `negligible` (under 0.05 points, with
  confidence) or `none` (the game can't reach the Bears in any season, from
  the schedule alone). The NFC North goal has no line: it is `matters` for
  any pick and `negligible` otherwise.
- `why`: the plain-language explanation, never empty. For a pick, the
  rivalry, then each measured channel (biggest first, up to four) with the
  team it favors and its size, then a net line when channels pull opposite
  ways. For "doesn't matter", a line naming the game and how small the
  difference is, then the channels that did show up or, if none did, the
  open links (who is still in the race, who is on the Bears' schedule). For
  `none`, why nothing can change.
- `channels`: the measured channels behind `why`, biggest first, each
  clearly bigger than its noise. Each has `channel` (`division_title`,
  `wild_card`, `seed`, `playoff_path` or `super_bowl_opponent`), `cause`
  (for the first three: `record`, `coin flip` or the tiebreaker step whose
  result differed, such as `strength of victory`; null otherwise),
  `favors` (the team whose win it helps), `share` (its size, as a
  probability), `std_error` and `seasons` (the share of simulated seasons
  in that channel and cause). Empty for `nfc_north`.

## Example

```json
{
  "generated_at": "2026-11-26T14:47:09Z",
  "season": 2026,
  "week": 12,
  "simulations": 100000,
  "latest_game": { "id": "2026_12_GB_LA", "gameday": "2026-11-25", "away": "GB", "home": "LA", "away_score": 13, "home_score": 26 },
  "team": "CHI",
  "focus": {
    "record": { "wins": 8, "losses": 2, "ties": 0 },
    "odds": { "playoffs": 0.907, "divisional_round": 0.580, "conference_game": 0.301, "super_bowl": 0.145, "champion": 0.068 },
    "division_lead": 1.0,
    "division_rival": "DET"
  },
  "banner": {
    "headline": "Bears win! Chicago 39, Lions 35.",
    "lines": ["Alone in first place in the NFC North, a game ahead of the Lions.", "Super Bowl odds up from 7.8% to 11%."]
  },
  "nfc_north_divisional": 0.0006,
  "games": [
    {
      "id": "2026_12_CHI_DET", "gameday": "2026-11-26", "away": "CHI", "home": "DET",
      "home_win_probability": 0.569,
      "champion": {
        "if_home_wins": 0.055, "if_away_wins": 0.083, "swing": -0.028, "std_error": 0.001, "root_for": "CHI", "tier": "matters",
        "why": [
          "The Bears' own game: a win over the Lions, a division rival, counts twice in the NFC North race.",
          "NFC North title, on record: a Bears win helps the Bears by 1.6 Super Bowl points."
        ],
        "channels": [
          { "channel": "division_title", "cause": "record", "favors": "CHI", "share": 0.0157, "std_error": 0.0008, "seasons": 0.363 }
        ]
      },
      "playoffs": { "if_home_wins": 0.864, "if_away_wins": 0.963, "swing": -0.099, "std_error": 0.0009, "root_for": "CHI", "tier": "matters", "why": ["..."], "channels": [] },
      "nfc_north": { "if_home_wins": 0.0007, "if_away_wins": 0.0005, "swing": 0.0002, "std_error": 0.0001, "root_for": null, "tier": "negligible", "why": ["..."], "channels": [] }
    }
  ]
}
```

## The archive

The site also serves past weeks' guides for the archive
([plan](plans/2026-11-archive.md)), each in this same format:

- `archive/<season>-week-<nn>.json`, such as `archive/2026-week-07.json`:
  the guide `bear_down --week <n> --season <year>` writes, so its `week` is
  that week.
- `archive/index.json`: the weeks there are, as
  `{"weeks": [{"season": 2026, "week": 7}, ...]}`. Order doesn't matter.
  The page reads it (`ArchiveIndex` in `web/src/data.rs`), lists the weeks
  newest first without the current one, and shows no picker when the file
  is missing or isn't valid.

## Running the CLI

From `bear_down/`:

```sh
cargo run --release -p bear_down_cli -- --out web/public/rooting.json --teams-dir web/public/teams
```

`--team <abbr>` writes another team's guide to `--out` instead of the
Bears', and `--teams-dir <dir>` writes every team but that one to
`<dir>/<team>.json`, on as many threads as the machine has cores.

It downloads nflverse `games.csv` (decision 0006) unless `--games` gives a
local path or another URL, and runs 100,000 simulations unless
`--simulations` says otherwise. Games close to the 0.05-point line get up to
four times as many. A weekly build takes under a minute in a release build
for one team, and about four minutes for all 32 on four cores.
