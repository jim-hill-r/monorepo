# 0009. Cloudflare Pages through the repo's deploy, rebuilt nightly

- Status: Accepted
- Date: 2026-11-26
- Supersedes: the hosting and schedule parts of [0001](0001-static-site-rebuilt-weekly.md)

## Context

Decision 0001 planned Cloudflare Workers static assets and a bear_down-only
GitHub Actions workflow that rebuilds every Tuesday. Since then the repo has
a shared deploy path: every project with a `cloudflare/` folder deploys to
Cloudflare Pages through `mise run deploy` (standards/tasks/cloudflare-pages.toml),
push-main CI/CD deploys it when a change lands on `main`, and Nightly CD
rebuilds and deploys every such project each night. cookbook and pane already
use it, and the repo-wide Cloudflare secrets are in place.

## Decision

- bear_down deploys to Cloudflare Pages from `bear_down/cloudflare/`, like
  cookbook and pane, at <https://jimhillr-bear-down.pages.dev> (Pages project
  `jimhillr-bear-down`, following `jimhillr-cookbook`).
- Its `release` task runs the simulation (`bear_down_cli`, which downloads
  nflverse `games.csv`) into `web/public/rooting.json` and then bundles the
  web app. The deploy uploads `web/dist/public`.
- There is no bear_down workflow and no Tuesday schedule. Push-main deploys
  changes, and Nightly CD rebuilds every night.
- The first deploy creates the Pages project through
  `standards/scripts/free-tiers.sh pages-project`, which only creates it
  inside the free tier budget (standards/docs/free-tiers.md).

## Consequences

- The odds refresh within a day of every game, including Thursday and Sunday
  games, not only on Tuesdays.
- No new workflow or secret: the deploy uses the repo's existing Cloudflare
  token and account id.
- Each nightly run downloads `games.csv` once and runs one simulation, well
  inside GitHub Actions and Cloudflare Pages free limits (one deploy a night
  against 500 builds a month).
- A custom domain can still replace the `pages.dev` address later.
