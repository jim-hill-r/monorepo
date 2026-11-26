
# Priority Issues

# Backlog

Feedback on every change: see
[standards/docs/plans/2026-11-deploy-feedback.md](standards/docs/plans/2026-11-deploy-feedback.md).
They build on the mise workflows from PR #570 (`mise-projects.yml`,
`push-main.yml`, `pull-request-ci.yml`).

- TODO (agent-generated): CD cleanup. Give each deploying project a `production` GitHub environment with its URL, write the deployed URL to the job summary, record the URL as `DEPLOY_URL` in the project's `mise.toml`, and add a `workflow_dispatch` redeploy of one project at a given ref for rollbacks.
- TODO (agent-generated): Preview deploys. Add a `preview` task to `standards/tasks/cloudflare-pages.toml` (`wrangler pages deploy --branch pr-<n>`), run it in PR CI for affected deploy projects on same-repo PRs, and keep one sticky PR comment with the preview URLs.
- TODO (agent-ignore): Add the Cloudflare Pages preview domains to the Auth0 allowed callback URLs for cookbook and cahokia. Needs Jim in the Auth0 dashboard.
- TODO (agent-generated): Smoke UAT. Tag a few Playwright tests `@smoke` in cookbook/web and cahokia/web, let `playwright.config.ts` use `BASE_URL` and record video, screenshots and traces for smoke runs, and add a `smoke` task to the web task files.
- TODO (agent-generated): Run the smoke tests against each PR preview and against production after each deploy, publish the Playwright HTML report to a `uat-reports` Cloudflare Pages project, and link it from the sticky PR comment.
- TODO (agent-generated): Add a UAT section (what changed for the user, what to click on the preview) to the PR template and the `open-pull-request` skill, embed the key Playwright screenshots inline in the sticky PR comment, and change the screenshot rule in `standards/docs/documentation.md` to "link the recorded UAT report".
- TODO (agent-generated): Add a `smoke` task for cahokia/desktop that launches the release build under `xvfb-run`, screenshots the window, records a short `ffmpeg` video and fails if the app exits early.
- TODO (agent-generated): Add a nightly job that builds cahokia/mobile for Android and takes one screenshot in the Android emulator action.
- TODO (agent-generated): Add Pages deploys and `@smoke` Playwright tests for blueeel and sparkhill/webapp, matching cookbook, pane and cahokia.
- TODO (agent-generated): CLI UAT. Add `trycmd` transcript tests (`--help`, one happy path per main command, one common error) to homer, cookbook/recipe_gen and my_lang, and a `smoke` task in `standards/tasks/rust.toml` that runs them against the release binary.
- TODO (agent-generated): CLI releases. Add a workflow that builds Linux, macOS and Windows binaries for each CLI on a version tag or `workflow_dispatch`, attaches them to a GitHub Release, and runs the smoke task against the Linux binary.
- TODO (agent-generated): Render a `vhs` tape per CLI (`docs/demo.tape`) to a GIF when a PR changes user-facing output, and link it from the sticky PR comment.
- TODO (agent-generated): Bevy UAT. Add a scripted mode to controlled_chaos (fixed timestep, seeded RNG, timed inputs, screenshots at named frames, non-zero exit on panic) and a `smoke` task in `standards/tasks/bevy-game.toml` that runs it under `xvfb-run` and records the display with `ffmpeg`.
- TODO (agent-generated): Check whether controlled_chaos builds for `wasm32-unknown-unknown`; if it does, deploy the web build as a Pages preview per PR and to production on main.
- TODO (agent-generated): Monthly full run. Add `monthly-uat.yml` (first Tuesday of each month plus `workflow_dispatch`) that runs `ci`, `release`, `deploy` and `smoke` for every project, including on-hold ones, and publishes the combined UAT report to `uat-reports` under `monthly-<yyyy-mm>`.
- TODO (agent-generated): Give every shared task file a default `smoke` task that fails with "no smoke test yet", so projects without UAT show up in the monthly report.
- TODO (agent-generated): Add a scheduled agent task after the monthly run that records a `FIX` in the `ISSUES.md` of each failed project in one draft PR, and flag any project whose last successful UAT is older than 31 days.
- TODO (agent-ignore): Create the PostHog Cloud project and add a read-only PostHog personal API key as a repo secret. Needs Jim.
- TODO (agent-generated): Add PostHog session replay to each production web app (production host only, inputs masked, console and network capture on) with a privacy note on each site.
- TODO (agent-generated): Send PostHog gameplay events (session start, level or turn reached, quit, panic message) from the controlled_chaos web build.
- TODO (agent-generated): Add a `review-user-sessions` agent skill that reads the past week's PostHog sessions with errors, rage clicks or dead clicks and records `FIX`/`TODO` items with recording links in the right `ISSUES.md`, and ask Jim to schedule it weekly.
- TODO (agent-ignore): Create a Honeycomb account on the free tier and add `HONEYCOMB_API_KEY` as a repo secret. Needs Jim.
- TODO (agent-generated): Send GitHub Actions CI/CD runs to Honeycomb as traces, one span per job and step.
- TODO (agent-generated): Post a Honeycomb deploy marker (project, commit, PR) from every production deploy.
- TODO (agent-generated): Add Honeycomb's OpenTelemetry web SDK to each web app for page loads, Core Web Vitals and uncaught errors, using an ingest-only key.
- TODO (agent-generated): Add an opt-in `tracing` OTLP exporter to each CLI that sends errors and command timings to Honeycomb only when `HONEYCOMB_API_KEY` is set.
- TODO (agent-generated): Export `tracing` spans over OTLP to Honeycomb from any Rust service with a backend, using `tracing-opentelemetry`.
- TODO (agent-generated): Add a Honeycomb trigger that alerts when browser errors rise after a deploy marker.

# Priority Projects
- cast
- cookbook
- homer
- controlled_chaos
- royalty
- sparkhill

# On Hold Projects
- luggage
- starcraft
- profiles