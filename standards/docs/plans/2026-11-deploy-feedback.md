# Plan: feedback on every change

Started 2026-11-26. The decision behind this plan is
[../decisions/0001-deploy-feedback-loop.md](../decisions/0001-deploy-feedback-loop.md).
The work items are `TODO` lines in the root [ISSUES.md](../../../ISSUES.md)
under `# Backlog`, in the order below.

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
| Dioxus web app | cookbook/web, pane/web, cahokia/web, sparkhill/webapp, blueeel, bear_down (coming) | Preview URL, Playwright `@smoke` with screenshots and video | `@smoke` against production | PostHog replay, Honeycomb browser telemetry |
| Perseus site | sparkhill/website, profiles/jimhillr (not deployed yet) | Same as web apps once deployed (`perseus export` is static, so Pages works) | Same | Same |
| Dioxus desktop app | cahokia/desktop | Launch under xvfb, screenshot and video of the window | Smoke run against the release bundle | None yet |
| Dioxus mobile app | cahokia/mobile | Covered by cahokia/web through the shared `ui` crate | Nightly Android emulator run with a screenshot; iOS not covered | None yet |
| CLI | homer, cookbook/recipe_gen, my_lang (compiles and runs sample programs) | `trycmd` transcript tests, `vhs` GIF when output changes | Smoke session against the released binary | Opt-in `tracing` export to Honeycomb |
| Bevy game | controlled_chaos (starcraft on hold) | Playable web preview if it builds for `wasm32`, scripted xvfb run with screenshots and video | Scripted run against the release build | PostHog gameplay events from the web build |
| HTTP API | luggage (on hold, not deployed) | HTTP smoke test of the OpenAPI routes against `cargo run` | Same test against the deployed URL | Honeycomb traces through `tracing` |
| Library | auth_sdk, cahokia/ui, cookbook/core, profiles/content_provider, luggage crates | Unit tests plus the consumers' CI (`affected-by`) | Nothing to release | Through the apps that use it |
| Content and docs | cookbook/content and data_md, athenaeum, bogan, psy_ents, royalty, bear_down docs | Markdown and link checks; cookbook content is checked by building and smoke testing the site | Nothing to release | None |
| Going away with Cast | cast/cli, cast/vscode_ext, the standards CLI, monorepo/workflow_tests | Existing CI until removed | None | None |

Honeycomb adds the numbers (CI times, errors, page performance) next to those
recordings.

## Where we start

- Web apps: cookbook, pane and cahokia deploy to Cloudflare Pages through a
  `*/cloudflare` project that runs `wrangler pages deploy`. sparkhill has a
  web app but no deploy yet. bear_down will serve Workers static assets.
- [PR #570](https://github.com/jimhillr/monorepo/pull/570) (mise migration
  step 2) replaces the artifacts-triggered CD with `push-main.yml`, which runs
  `mise run ci` then `mise run deploy` in each affected project through the
  reusable `mise-projects.yml`. That PR owns the workflow files, so all
  workflow changes below start after it merges and build on it.
- Playwright suites exist for cookbook/web and cahokia/web. Their configs
  always start a local `dx serve` and record nothing.
- No observability, analytics or replay anywhere.

## Phase 1: clean up CD (after #570 merges)

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
- A failed production smoke run fails `push-main.yml`. The hourly PR triage
  task, or the next agent to see it, adds a `FIX` to the project's
  `ISSUES.md` and offers a rollback.

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
  main, so the PR comment has a playable link.
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

## Phase 3e: monthly full run

Every project must be built, released or deployed, and UAT tested at least
once a month, even with no changes (Jim, 2026-11-26).

- Add `monthly-uat.yml`, scheduled for the first Tuesday of each month and
  runnable by hand (`workflow_dispatch`). It reuses `mise-projects.yml` with
  every project (empty `base`), and runs `ci`, `release`, `deploy` where a
  project has one, then `smoke`.
- Publish the combined UAT report (screenshots, videos, transcripts) to the
  `uat-reports` Pages project under a `monthly-<yyyy-mm>` branch and link it
  from the run summary.
- Every project needs a real `smoke` task for this to mean anything. Until a
  project has one, the shared task file's default fails with "no smoke test
  yet", so the gap shows in the report instead of passing quietly.
- A scheduled agent task runs after the monthly workflow: it reads the
  results, adds a `FIX` to the `ISSUES.md` of each project that failed, and
  opens one draft PR with those items and a short summary.
- A check in `//:audit` (or the monthly run itself) lists any project whose
  last successful UAT is older than 31 days.
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
  recording links into the right `ISSUES.md` in one draft PR. A short summary
  of the week goes into the PR body so Jim can watch the two or three
  recordings that matter.
- When a PR fixes something found this way, its body links the recording.
- Games: the web build sends a few PostHog events (session start, level or
  turn reached, quit, panic message). Replay of the game canvas is not
  attempted; the review skill reads these events alongside the web sessions.
- CLIs: no tracking by default. Setting `HONEYCOMB_API_KEY` in the
  environment opts in to sending errors and command timings (Phase 5).

## Phase 5: Honeycomb

- CI/CD traces: send each workflow run to Honeycomb as a trace (one span per
  job and step) so slow or flaky CI shows up. Needs `HONEYCOMB_API_KEY`.
- Deploy markers: each production deploy posts a Honeycomb marker with the
  project, commit and PR, so graphs show when a change landed.
- Browser telemetry: add Honeycomb's OpenTelemetry web SDK to each web app
  for page loads, Core Web Vitals and uncaught errors, with an ingest-only
  key.
- CLIs: an opt-in `tracing` OTLP exporter, on only when the user sets
  `HONEYCOMB_API_KEY`, for errors and command timings.
- Games: frame-time and load-time spans from the scripted CI run, so a slow
  change shows up as a regression.
- Rust services: any server code (for example `auth_sdk` users with a
  backend) exports its `tracing` spans over OTLP with `tracing-opentelemetry`.
- One trigger: alert Jim if browser errors jump after a deploy marker.

## Cost

Free tier limits as of 2026-11; check them before relying on them.

| Piece | Tool | Cost at our traffic |
| --- | --- | --- |
| Previews and report hosting | Cloudflare Pages | Free |
| CLI and game releases | GitHub Releases | Free |
| Terminal recordings | `vhs`, `trycmd` | Free (open source) |
| Monthly full run | GitHub Actions | One full build of every project a month; the largest single use of minutes, still free on public repos |
| Smoke runs | GitHub Actions | Free on public repos; about a minute per app per run otherwise |
| Session replay | PostHog Cloud | Free up to 5,000 recordings and 1M events a month |
| Observability | Honeycomb | Free up to 20M events a month |
| Private previews, if ever needed | Cloudflare Access | Free up to 50 users |

## Not doing (yet)

- Visual regression diffs against a baseline. Revisit once the smoke
  screenshots are stable.
- A paid replay or monitoring plan. Revisit only if a free tier runs out.
- Previews for fork or dependabot PRs.
