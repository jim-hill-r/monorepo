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

## Phase 3: recorded smoke UAT

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
- The PR template and `open-pull-request` skill gain a short **UAT** section:
  what changed for the user and what the reviewer should click on the
  preview. A reviewer approves only after opening the preview and the report.
- A failed production smoke run fails `push-main.yml`. The hourly PR triage
  task, or the next agent to see it, adds a `FIX` to the project's
  `ISSUES.md` and offers a rollback.

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

## Phase 5: Honeycomb

- CI/CD traces: send each workflow run to Honeycomb as a trace (one span per
  job and step) so slow or flaky CI shows up. Needs `HONEYCOMB_API_KEY`.
- Deploy markers: each production deploy posts a Honeycomb marker with the
  project, commit and PR, so graphs show when a change landed.
- Browser telemetry: add Honeycomb's OpenTelemetry web SDK to each web app
  for page loads, Core Web Vitals and uncaught errors, with an ingest-only
  key.
- Rust services: any server code (for example `auth_sdk` users with a
  backend) exports its `tracing` spans over OTLP with `tracing-opentelemetry`.
- One trigger: alert Jim if browser errors jump after a deploy marker.

## Cost

Free tier limits as of 2026-11; check them before relying on them.

| Piece | Tool | Cost at our traffic |
| --- | --- | --- |
| Previews and report hosting | Cloudflare Pages | Free |
| Smoke runs | GitHub Actions | Free on public repos; about a minute per app per run otherwise |
| Session replay | PostHog Cloud | Free up to 5,000 recordings and 1M events a month |
| Observability | Honeycomb | Free up to 20M events a month |
| Private previews, if ever needed | Cloudflare Access | Free up to 50 users |

## Not doing (yet)

- Visual regression diffs against a baseline. Revisit once the smoke
  screenshots are stable.
- A paid replay or monitoring plan. Revisit only if a free tier runs out.
- Previews for fork or dependabot PRs.
