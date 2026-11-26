# 0001. Static site, simulated at build time, rebuilt weekly

- Status: Accepted; hosting and schedule superseded by [0009](0009-cloudflare-pages-nightly-cd.md)
- Date: 2026-11-25

## Context

The site needs fresh odds each week but should stay as simple and cheap to run
as possible. A server that simulates on request would need hosting, caching
and monitoring for numbers that only change when games finish.

## Decision

- There is no backend. All simulation runs at build time.
- The project is a Cargo workspace with three crates, following the naming
  standard (`<project>_<dir>` package names):
  - `core` (`bear_down_core`): data loading, Elo, standings, tiebreakers,
    simulation and rooting analysis. No I/O beyond parsing.
  - `cli` (`bear_down_cli`): downloads the data, runs the simulation and
    writes `rooting.json`.
  - `web` (`bear_down_web`): a Dioxus web app, matching cookbook and pane,
    that fetches `rooting.json` from the same origin at page load and renders
    it. A sample `rooting.json` lets the page be developed without running a
    simulation.
- The built site is served as Cloudflare Workers static assets on a
  `workers.dev` address until a custom domain is chosen.
- A GitHub Actions workflow builds and deploys every Tuesday morning, after
  Monday Night Football, and can also be run by hand (`workflow_dispatch`).
  There is no extra rebuild after Thursday games for now.
- Build output is not committed. CI builds and deploys directly, as in the
  repo-wide mise migration.

## Consequences

- Hosting is free and has nothing to keep running.
- The page is stale between a Thursday or Sunday game and the next Tuesday
  build. The page shows when it was generated and which games it includes.
- The deploy workflow needs a Cloudflare API token stored as a GitHub secret.
- Because the web crate only reads JSON, it can be swapped for plain HTML
  later without touching the simulator.
