# Plan: feedback on every change

Started 2026-11-26. The decision behind this plan is
[../decisions/0001-deploy-feedback-loop.md](../decisions/0001-deploy-feedback-loop.md).
The work items are issue files in the root [issues/](../issues/) with
`epic: Feedback on every change`, in the order below.

## Goal

Every change should come back with evidence of how it behaves for a real
person, at three moments:

1. **Before merge:** a preview URL, plus screenshots and a video of the smoke
   test running against it.
2. **Right after deploy:** the same smoke test against production, recorded,
   with a red run if it fails.
3. **After people use it:** recordings of real sessions, reviewed every week,
   turned into `FIX` and `TODO` items.

## Deliverable types

Inventory of every project with a `mise.toml`, 2026-11-26.

| Type | Projects | Before merge | After release | Real use |
| --- | --- | --- | --- | --- |
| Dioxus web app | cookbook/web, pane/web, cahokia/web, sparkhill/webapp, blueeel, bear_down (coming) | Preview URL, Playwright `@smoke` with screenshots and video | `@smoke` against production | PostHog replay, Cloudflare Web Analytics, errors to the `telemetry` Worker |
| Perseus site | sparkhill/website, profiles/jimhillr (not deployed yet) | Same as web apps once deployed (`perseus export` is static, so Pages works) | Same | Same |
| Dioxus desktop app | cahokia/desktop | Launch under xvfb, screenshot and video of the window | Smoke run against the release bundle | None yet |
| Dioxus mobile app | cahokia/mobile | Covered by cahokia/web through the shared `ui` crate | Nightly Android emulator run with a screenshot; iOS not covered | None yet |
| CLI | homer, cookbook/recipe_gen, my_lang (compiles and runs sample programs) | `trycmd` transcript tests, `vhs` GIF when output changes | Smoke session against the released binary | Opt-in reports to the `telemetry` Worker |
| Bevy game | controlled_chaos (starcraft on hold) | Playable web preview if it builds for `wasm32`, scripted xvfb run with screenshots and video | Scripted run against the release build | PostHog gameplay events from the web build |
| HTTP API | luggage (on hold, not deployed) | HTTP smoke test of the OpenAPI routes against `cargo run` | Same test against the deployed URL | Workers Logs, or `tracing` output to the `telemetry` Worker |
| Library | auth_sdk, cahokia/ui, cookbook/core, profiles/content_provider, luggage crates | Unit tests plus the consumers' CI (`affected-by`) | Nothing to release | Through the apps that use it |
| Content and docs | cookbook/content and data_md, athenaeum, bogan, psy_ents, royalty, bear_down docs | Markdown and link checks; cookbook content is checked by building and smoke testing the site | Nothing to release | None |
| Going away with Cast | cast/cli, cast/vscode_ext, the standards CLI, jimhillr/workflow_tests | Existing CI until removed | None | None |

Cloudflare adds the numbers (errors, page performance, deploys) next to those
recordings.

## Vendors

Jim asked on 2026-11-26 to use Cloudflare wherever it can do the job, to keep
the number of vendors down. The plan uses three: GitHub (code, CI,
releases), Cloudflare (hosting, previews, reports, analytics, logs) and
PostHog (real-user replay only).

| Need | Cloudflare option | Used? |
| --- | --- | --- |
| Previews and production hosting | Pages and Workers static assets | Yes |
| UAT report hosting | Pages project `uat-reports` (or R2) | Yes |
| Real-user analytics and performance | Web Analytics (free JS beacon) | Yes |
| Errors, events, deploy events | A `telemetry` Worker writing to Analytics Engine and Workers Logs | Yes |
| Logs and traces for Workers | Workers Observability (free: 0.5 GB a day, 7-day retention) | Yes |
| Headless browser for smoke tests | Browser Run (Playwright; free: 10 browser minutes a day) | Later, for a daily production check from a Cron Worker. Its recordings are DOM replays, not video, so PR evidence stays on GitHub Actions |
| Replay of real visitors | None. Browser Run records only sessions we drive | PostHog. A home-made rrweb recorder storing to R2 is the fallback if PostHog is dropped |
| OTLP export to another tool | Workers Paid ($5 a month) | Not needed |

## Where we start

- Web apps: cookbook, pane and cahokia deploy to Cloudflare Pages through a
  `*/cloudflare` project that runs `wrangler pages deploy`. sparkhill has a
  web app but no deploy yet. bear_down will serve Workers static assets.
- [PR #570](https://github.com/jimhillr/monorepo/pull/570) (mise migration
  step 2, merged 2026-11-26) replaced the artifacts-triggered CD with
  `push-main.yml`, which runs `mise run ci` then `mise run deploy` in each
  affected project through the reusable `mise-projects.yml`. All workflow
  changes below build on those files.
- Playwright suites exist for cookbook/web and cahokia/web. Their configs
  always start a local `dx serve` and record nothing.
- No observability, analytics or replay anywhere.

## Phase 1: clean up CD

Done on 2026-11-26 except the sparkhill and bear_down wiring, which waits for
their first deploy. How it works is in [../mise.md](../mise.md#deploys).

- Give each deploying project a `production` GitHub environment with its URL,
  so every deploy shows on the commit and in the repo's Deployments list.
- Each deploy job writes the deployed URL, commit and project to the job
  summary.
- Add a `workflow_dispatch` input to `push-main.yml` that redeploys one
  project at a given ref, so a bad deploy can be rolled back from GitHub.
- Record each project's production URL in one place (its `mise.toml` `[env]`
  as `DEPLOY_URL`) so smoke tests and comments can find it.
- Wire up sparkhill and bear_down the same way when they first deploy.

## Phase 2: preview deploy per PR

- Add a `preview` task to `standards/tasks/cloudflare-pages.toml` that runs
  `wrangler pages deploy --branch pr-$PR_NUMBER` and prints the alias URL
  (`https://pr-<n>.<project>.pages.dev`). Workers projects use
  `wrangler versions upload` with a preview alias.
- In PR CI, run `mise run preview` for affected deploy projects, only for PRs
  from this repo (forks and dependabot have no secrets).
- One sticky PR comment per PR, updated in place, lists each preview URL.
- Add the preview domains to the Auth0 allowed callback URLs for cookbook and
  cahokia, or sign-in fails on previews.

## Phase 3a: website smoke UAT

Progress (2026-11-27): cookbook runs its `@smoke` tests against production
after each deploy, and the cd evals read them, so COOK-005 (accessibility)
has a production check. The `smoke` task lives in the deploy task file
(`cloudflare-pages.toml`, through `standards/scripts/evals.sh smoke` and the
project's `SMOKE_DIR`), not the web task files, since the deploy job runs in
the deploying project. Still to do: the other web apps, previews, and the
`uat-reports` site ([evals.md](../evals.md#after-deploy), the open issues in
`issues/`).

- Tag a handful of existing Playwright tests `@smoke` in each web app: the
  home page loads, main navigation works, one core flow per app, no console
  errors. Each app's smoke run should take about a minute in one browser.
- Make `playwright.config.ts` use `BASE_URL` when set and skip `webServer`,
  and record `video: 'on'`, `screenshot: 'on'` and `trace: 'on'` for smoke
  runs only.
- Add a `smoke` task to the web task files: `npx playwright test --grep
  @smoke --project chromium`.
- Run it against the preview in PR CI and against production right after each
  deploy on main.
- Publish the Playwright HTML report (with videos) to a `uat-reports`
  Cloudflare Pages project, branch `pr-<n>` for PRs and `main` for
  production. The sticky PR comment links to it; the Actions artifact is kept
  for 14 days as a backup.
- Playwright screenshots and video become the standard evidence on every PR
  that changes UI (Jim, 2026-11-26). `documentation.md` changes from "take
  screenshots" to "link the recorded UAT report", and the sticky PR comment
  embeds the key screenshots inline from the `uat-reports` site so reviewers
  see them without clicking.
- The PR template and `open-pull-request` skill gain a short **UAT** section:
  what changed for the user and what the reviewer should click on the
  preview. A reviewer approves only after opening the preview and the report.
- A failed production smoke run fails `push-main.yml`. The PR triage
  task, or the next agent to see it, adds a `FIX` to the project's
  `issues/` and offers a rollback.

## Phase 3b: CLI UAT

- Add `trycmd` transcript tests to each CLI: `--help`, one happy path per
  main command on fixture data, and one common mistake with its error
  message. These run in `mise run ci`, so they gate every PR.
- Add a `smoke` task to `standards/tasks/rust.toml` for projects that ship a
  binary: build in release mode and run the same transcripts against it.
- When a PR changes user-facing output, render a `vhs` tape kept in the
  project (`docs/demo.tape`) to a GIF, publish it with the UAT report and
  link it from the sticky PR comment.
- Replace committed artifacts with a release workflow: on a version tag (or a
  `workflow_dispatch`), build Linux, macOS and Windows binaries and attach
  them to a GitHub Release, then run the smoke task against the Linux
  binary.

## Phase 3c: Bevy game UAT

- Add a scripted mode to controlled_chaos, behind a `--script <file>` flag or
  environment variable: fixed timestep, seeded RNG, a list of timed inputs,
  screenshots at named frames with Bevy's screenshot API, then exit with a
  non-zero code on a panic or a failed check.
- Add a `smoke` task to `standards/tasks/bevy-game.toml` that runs the
  script under `xvfb-run`, records the virtual display with `ffmpeg`, and
  writes screenshots and the video next to the UAT report.
- Check whether controlled_chaos builds for `wasm32-unknown-unknown`. If it
  does, deploy the web build as a Pages preview per PR and to production on
  main, so the PR comment has a playable link. Done for production
  (2026-11-27): it builds, and `bevy-game.toml`'s `deploy` task publishes it
  to `jimhillr-controlled-chaos.pages.dev` from main. The per-PR preview is
  part of the phase 2 preview issue, which covers `bevy-game.toml` too.
- Native builds go to a GitHub Release, as for CLIs.

## Phase 3d: native apps and the API

- cahokia/desktop: a `smoke` task that launches the release build under
  `xvfb-run`, waits for the window, takes a screenshot and records a short
  `ffmpeg` video, and fails if the process exits early.
- cahokia/mobile: rely on cahokia/web for shared UI. Add a nightly job that
  builds the Android target and runs it in the free Android emulator action
  for one screenshot. Skip iOS until there is a reason to pay for macOS
  minutes.
- luggage (when it comes off hold): an HTTP smoke test that walks the
  OpenAPI routes against `cargo run`, then against the deployed URL.
- Perseus sites: when sparkhill/website or profiles/jimhillr first deploys,
  give it a Pages project and reuse the website path above.

## Phase 3e: nightly full run

Every project runs full CI and CD every night, even with no changes (Jim,
2026-11-26, replacing the monthly run planned earlier that day). Only the
latest version of main that passed CI is deployed.

- Done: `nightly-ci.yml` runs `ci` for every project at 2 AM Pacific, and
  `nightly-cd.yml` runs `ci` then `deploy` for every deploying project at
  2:30 AM, both through `mise-projects.yml`, with a README badge each. Each deploy first runs
  `standards/scripts/deploy-check.sh`, which skips it when a newer commit on
  main changes the project.
- Add a `smoke` step after `deploy` once projects have smoke tests, and
  publish the combined UAT report (screenshots, videos, transcripts) to the
  `uat-reports` Pages project under a `nightly` branch, linked from the run
  summary.
- Every project needs a real `smoke` task for this to mean anything. Until a
  project has one, the shared task file's default fails with "no smoke test
  yet", so the gap shows in the report instead of passing quietly.
- A scheduled agent task runs after the nightly workflow: it reads the
  results, adds a `FIX` issue file to the `issues/` of each project that failed, and
  opens one draft PR with those items and a short summary.
- A check lists any project whose last successful UAT is older than 2 days.
- On-hold projects are included. If one cannot build, it gets a `FIX`, not a
  skip.

## Phase 4: real user sessions (PostHog)

Jim chose PostHog Cloud on its free tier on 2026-11-26.

- Add the PostHog snippet to each web app's HTML shell, loaded only when the
  host is the production domain. Mask all inputs and any text marked
  sensitive, respect Do Not Track, and add a one-line privacy note to each
  site.
- Keep the public project key in each app's config, not as a secret.
- Turn on PostHog's console log and network capture so recordings show
  errors next to what the user did.
- Add an agent skill, `review-user-sessions`, and a weekly scheduled agent
  task that runs it. It reads the past week's recordings and events through
  the PostHog API (sessions with exceptions, rage clicks, dead clicks or quick
  exits), groups them by app and page, and writes `FIX` and `TODO` items with
  recording links as issue files in the right `issues/` folder in one draft PR. A short summary
  of the week goes into the PR body so Jim can watch the two or three
  recordings that matter.
- When a PR fixes something found this way, its body links the recording.
- Games: the web build sends a few PostHog events (session start, level or
  turn reached, quit, panic message). Replay of the game canvas is not
  attempted; the review skill reads these events alongside the web sessions.
- CLIs: no tracking by default. Setting `TELEMETRY_URL` in the environment
  opts in to sending errors and command timings (Phase 5).

## Phase 5: Cloudflare observability

- Turn on Cloudflare Web Analytics for every production site (one switch per
  Pages project, or the JS beacon for Workers sites). No cookies, so no
  consent banner change.
- Add a `telemetry` project: a small Rust Worker with one `POST /event`
  route that validates and rate-limits events, writes them to an Analytics
  Engine dataset, and logs them to Workers Logs. Event kinds: `error`
  (browser or game panic, with page and build), `game` (gameplay events),
  `cli` (opt-in reports) and `deploy` (project, commit, PR).
- Each web app reports uncaught errors and panics to it. Each production
  deploy posts a `deploy` event, so errors can be lined up with the change
  that caused them.
- CLIs post errors and command timings only when the user sets
  `TELEMETRY_URL` (opt-in).
- The weekly `review-user-sessions` skill also queries Analytics Engine
  through its SQL API: error counts per app and build, and whether errors
  rose after a deploy event. That replaces a separate alerting tool.
- Games: frame-time and load-time numbers from the scripted CI run go into
  the job summary, so a slow change shows up as a regression.
- Later, if needed: a daily production smoke check from a Cron Worker using
  Browser Run, inside its free 10 minutes a day.

## Cost

Free tier limits as of 2026-11; check them before relying on them.

| Piece | Tool | Cost at our traffic |
| --- | --- | --- |
| Previews and report hosting | Cloudflare Pages | Free |
| CLI and game releases | GitHub Releases | Free |
| Terminal recordings | `vhs`, `trycmd` | Free (open source) |
| Nightly full run | GitHub Actions, Cloudflare Pages direct upload | One full build and deploy of every project a night; the largest use of minutes, free on standard runners in a public repo, and direct uploads use no Pages builds |
| Smoke runs | GitHub Actions | Free on public repos; about a minute per app per run otherwise |
| Session replay | PostHog Cloud | Free up to 5,000 recordings and 1M events a month |
| Analytics | Cloudflare Web Analytics | Free |
| Errors, events, logs | Cloudflare Workers, Analytics Engine, Workers Observability | Free tier (0.5 GB a day of logs, 7-day retention) |
| Private previews, if ever needed | Cloudflare Access | Free up to 50 users |

## Not doing (yet)

- Visual regression diffs against a baseline. Revisit once the smoke
  screenshots are stable.
- A paid replay or monitoring plan. Revisit only if a free tier runs out.
- Previews for fork or dependabot PRs.
