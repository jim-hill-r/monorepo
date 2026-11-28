# Plan: an archive of past weeks' guides

- Status: proposed (2026-11-27), phases 1 and 2 done, phase 3 next
- Issue: the backlog's "Archive of past weeks' guides"
  ([0008](../decisions/0008-v1-scope.md) left it out of v1)
- Builds on: [0001](../decisions/0001-static-site-rebuilt-weekly.md) and
  [0009](../decisions/0009-cloudflare-pages-nightly-cd.md) (a static site
  rebuilt nightly)

## Why

Only this week's guide is on the site. A fan can't look back at what the
guide said last week, or check whether the games it called "doesn't
matter" really didn't.

## Approach: rebuild past weeks, don't store them

Each past week's guide is rebuilt from nflverse `games.csv` on every
deploy, from the results before that week. Nothing is stored between
builds.

- **No storage.** CI never writes `main` (SEC-007), and the repo has no
  data branch for the site. Storing published `rooting.json` files would
  need a new data branch, or a database. Both go against "files before
  databases" for data that can be rebuilt.
- **Repeatable.** The simulation is seeded by season and week
  (`SimConfig::weekly`), so the same data and code give the same guide.
- **What it shows.** The archive shows what today's model says about a
  past week, from what was known then. It can differ a little from what the
  site showed that week, for example after the Elo tuning or a fix to the
  explanations. The archive page says so in one line.
- **Known then.** `Schedule::as_of(year, week)` clears every result from
  that week on and drops later playoff games, whose matchups would give
  the field away. It keeps betting lines only for that week and the next,
  about what nflverse publishes ahead (decision 0004's amendment).

The other option was to keep each week's published `rooting.json`. The
nightly deploy would fetch the live site's archive and add the new week.
That copy is exact, but it is lost if a deploy fails or Pages drops the
files, and it can't be rebuilt. It stays the fallback if Jim wants the
archive to match the published guides exactly.

## Cost

$0. A release build rebuilds a week in about 20 seconds (100,000
simulations, one core). Late in the season that adds about 6 minutes to
the nightly deploy, on free GitHub Actions minutes for a public repo.
Pages gains one JSON file of about 50 KB per week, far inside the free
plan's 20,000 files per site. Nothing new to sign up for.

## Phases

Each is one PR.

1. **Rebuild one past week.** `Schedule::as_of` in `bear_down_core`, and
   `bear_down --week <n> [--season <year>]` in the CLI. No visible change.
2. **The archive page.** The web app reads `archive/index.json` (season
   and week list) and `archive/<season>-week-<nn>.json`. It shows a week
   picker under the guide, with the past guide in the same layout and a
   "rebuilt with today's model" line. Includes Playwright tests and
   screenshots of the picker on a phone. The index is
   `{"weeks": [{"season": 2026, "week": 11}, ...]}` in any order; the page
   offers them newest first, leaving out the week the live guide shows, and
   shows no picker while there is no index (`docs/rooting-json.md`).
3. **Build the archive on deploy.** The `cloudflare` release task builds
   every past week of the current season into `web/public/archive/`, plus
   the index. Waits for the deploy path fix in PR #774, which changes the
   same task.

Voting stays on this week's guide only. Votes on old weeks would mix into
the A/B test's results.
