
# Priority Issues

# Backlog

Feedback on every change: see
[standards/docs/plans/2026-11-deploy-feedback.md](standards/docs/plans/2026-11-deploy-feedback.md).
They build on the mise workflows from PR #570 (`mise-projects.yml`,
`push-main.yml`, `pull-request-ci.yml`).

- TODO (agent-generated): Preview deploys. Add a `preview` task to `standards/tasks/cloudflare-pages.toml` (`wrangler pages deploy --branch pr-<n>`), run it in PR CI for affected deploy projects on same-repo PRs, and keep one sticky PR comment with the preview URLs.
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
- TODO (agent-generated): Nightly UAT. Add a `smoke` step after `deploy` in the nightly run (`nightly-cd.yml` through `mise-projects.yml`, plus a smoke-only pass for projects that don't deploy) for every project, including on-hold ones, and publish the combined UAT report to `uat-reports` under a `nightly` branch.
- TODO (agent-generated): Give every shared task file a default `smoke` task that fails with "no smoke test yet", so projects without UAT show up in the nightly report.
- TODO (agent-generated): Add a scheduled agent task after the nightly run that records a `FIX` in the `ISSUES.md` of each failed project in one draft PR, and flag any project whose last successful UAT is older than 2 days.
- TODO (agent-generated): Add PostHog session replay to each production web app (production host only, inputs masked, console and network capture on) with a privacy note on each site.
- TODO (agent-generated): Send gameplay events (session start, level or turn reached, quit, panic message) from the controlled_chaos web build to the `telemetry` Worker.
- TODO (agent-generated): Add a `review-user-sessions` agent skill that reads the past week's PostHog sessions with errors, rage clicks or dead clicks and records `FIX`/`TODO` items with recording links in the right `ISSUES.md`, and ask Jim to schedule it weekly.
- TODO (agent-generated): Add a `telemetry` Rust Worker project with a `POST /event` route that validates and rate-limits events (`error`, `game`, `cli`, `deploy`) and writes them to an Analytics Engine dataset and Workers Logs.
- TODO (agent-generated): Report uncaught errors and panics from each web app to the `telemetry` Worker, and post a `deploy` event (project, commit, PR) from every production deploy.
- TODO (agent-generated): Add opt-in error and timing reports from each CLI to the `telemetry` Worker, sent only when `TELEMETRY_URL` is set.
- TODO (agent-generated): Make the `review-user-sessions` skill query Analytics Engine for error counts per app and build and flag errors that rose after a deploy event.

Free tiers: see [standards/docs/free-tiers.md](standards/docs/free-tiers.md).

- TODO (agent-generated): Once the nightly run deploys every project (PR #583), make `nightly-ci.yml` call `all-free-tier-usage.yml` as its first job and make the `mise-projects.yml` job need it, so a nightly deploy never runs past a budget.
- TODO (agent-generated): When bear_down or the `telemetry` Worker first deploys, add a `free-tiers.sh worker` guard (count Workers and Cron Triggers in the account before `wrangler deploy`) and run it from the Workers deploy task, like `pages-project` for Pages.
- TODO (agent-generated): When PostHog is added, extend `free-tiers.sh usage` to read the month's event and recording counts from the PostHog API and warn at 80% of the free allowance.
- TODO (agent-generated): Cost approval process. Add a "Spending money" section to `standards/docs/free-tiers.md`: any expense (paid plan, add-on, domain, payment method) needs a decision record in `standards/docs/decisions/` with the monthly and yearly cost, why the free tier isn't enough, and how to cancel. The PR gets the `needs-human` label and only Jim merges it. Add a check to `free-tiers.sh check` that fails when a decision record says `Cost:` with a non-zero amount but is not marked `Approved by: Jim`.
- TODO (agent-generated): Cost ledger. Add `standards/docs/costs.md` listing every service, plan, monthly cost (all $0 today), payment method on file (none) and the decision record behind any paid line, and have the `usage` check fail if a service in the guards isn't listed there.
- TODO (agent-generated): Monthly cost review. Add a `review-costs` agent skill and ask Jim to schedule it on the 1st of each month. It runs All Free Tier Usage, compares the results with `standards/docs/costs.md`, records the month's usage in `standards/docs/costs/<yyyy-mm>.md`, and labels a PR `needs-human` when anything is above 80% of a free limit or not $0.
- TODO (agent-generated): Billing checks in the weekly usage workflow, where the vendor's API allows it. Use a separate read-only secret for each: GitHub's billing usage API (fail on any non-zero net amount for the month), and Cloudflare's account subscriptions API with a Billing Read token (fail on any paid subscription, Workers Paid included). Record in `free-tiers.md` which vendors can't be checked by API (PostHog, Auth0, RapidAPI) so the monthly review asks Jim about them.

# Priority Projects
- cookbook
- homer
- controlled_chaos
- royalty
- sparkhill

# On Hold Projects
- luggage
- starcraft
- profiles