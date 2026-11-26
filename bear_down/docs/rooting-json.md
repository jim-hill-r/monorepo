# rooting.json

The `bear_down` CLI writes `rooting.json`, and the web page fetches it from
the same origin at page load (decision 0001). The Rust types are in
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
| `bears.record` | object | `wins`, `losses`, `ties`. |
| `bears.odds` | object | `playoffs`, `divisional_round`, `conference_game`, `super_bowl` (reached it), `champion` (won it). |
| `nfc_north_divisional` | number | The chance all four NFC North teams reach the divisional round (decision 0007). |
| `games` | array | The current week's games still to be played, in schedule order. |

Each game has `id`, `gameday`, `away`, `home`, `home_win_probability` and one
swing object per goal:

- `champion`: the Bears winning the Super Bowl, the default goal (decision 0002).
- `playoffs`: the Bears making the playoffs.
- `nfc_north`: all four NFC North teams reaching the divisional round.

A swing object has `if_home_wins` and `if_away_wins` (the goal's odds after
each result), `swing` (`if_home_wins - if_away_wins`; negative means root for
the away team) and `root_for` (the team to root for, or null when the swing
is within two standard errors of the simulation's noise, shown as "doesn't
matter").

## Example

```json
{
  "generated_at": "2026-11-26T14:47:09Z",
  "season": 2026,
  "week": 12,
  "simulations": 100000,
  "latest_game": { "id": "2026_12_GB_LA", "gameday": "2026-11-25", "away": "GB", "home": "LA", "away_score": 13, "home_score": 26 },
  "bears": {
    "record": { "wins": 8, "losses": 2, "ties": 0 },
    "odds": { "playoffs": 0.907, "divisional_round": 0.580, "conference_game": 0.301, "super_bowl": 0.145, "champion": 0.068 }
  },
  "nfc_north_divisional": 0.0006,
  "games": [
    {
      "id": "2026_12_CHI_DET", "gameday": "2026-11-26", "away": "CHI", "home": "DET",
      "home_win_probability": 0.569,
      "champion": { "if_home_wins": 0.055, "if_away_wins": 0.086, "swing": -0.031, "root_for": "CHI" },
      "playoffs": { "if_home_wins": 0.864, "if_away_wins": 0.963, "swing": -0.099, "root_for": "CHI" },
      "nfc_north": { "if_home_wins": 0.0007, "if_away_wins": 0.0005, "swing": 0.0002, "root_for": null }
    }
  ]
}
```

## Running the CLI

From `bear_down/`:

```sh
cargo run --release -p bear_down_cli -- --out web/public/rooting.json
```

It downloads nflverse `games.csv` (decision 0006) unless `--games` gives a
local path or another URL, and runs 100,000 simulations unless
`--simulations` says otherwise. A weekly build takes a few seconds in a
release build.
