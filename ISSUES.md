
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

Eval engine: see
[standards/docs/plans/2026-11-eval-engine.md](standards/docs/plans/2026-11-eval-engine.md)
and decision 0003. Work the phases in order. Phases 0 and 1 depend on
nothing; later phases need the deploy-feedback items above. The pane pages
for it are in [pane/ISSUES.md](pane/ISSUES.md).

- TODO (agent-generated): Eval engine phase 0. Run `evals check` (in `evals/cli`; it already reports EVL-001 to EVL-004 and malformed files) from `mise run //:audit`, as warnings for now, without making every project's CI build the `evals` crates from scratch (for example, build it once in the audit job and cache it).
- TODO (agent-generated): Eval engine phase 0. Add `evals run --stage ci` for the `cargo` (test names prefixed with the requirement ID) and `playwright` (`@req:<ID>` in the title, JSON reporter) kinds, reading the reports the project's `ci` task already wrote, and `evals publish` writing the per-project job summary table and an `evals-results.jsonl` artifact.
- TODO (agent-generated): Eval engine phase 0. Draft `cookbook/docs/requirements.md` and `cookbook/evals.toml` from its README, the 2026-11-25 review deck's Goals slide and its `ISSUES.md`, tag the matching existing Playwright and Rust tests, and open the PR with the `needs-human` label for Jim to approve the requirements.
- TODO (agent-generated): Eval engine phase 1. Add an `evals` task to each shared task file in `standards/tasks/` and run it after `ci` in `mise-projects.yml` (PR CI, push to main, nightly CI) with stage `ci`.
- TODO (agent-generated): Eval engine phase 1. Make `evals publish` commit each run's results to the long-lived `evals-results` data branch (`results/<project>/<yyyy-mm>.jsonl` appended, `verdicts.json` recomputed, one commit per run, retry on a push race) with `contents: write` for that step only, add `evals verdicts`, and a yearly step that archives months older than 12 to `results/<project>/archive/<yyyy>.jsonl.gz`. No database. Add a repo check that eval workflows push only to `evals-results`, and note in `all-delete-branches.yml` that the branch must never be deleted.
- TODO (agent-generated): Eval engine phase 1. Draft `requirements.md` and `evals.toml` for pane and cahokia, tagging their existing tests, each in a `needs-human` PR.
- TODO (agent-generated): Eval engine phase 1. Once preview deploys exist, list the requirements whose evals a PR touched, with their status on the preview, in the sticky PR comment.
- TODO (agent-generated): Eval engine phase 2. Run `evals run --stage cd` after `deploy` and `smoke` in `mise-projects.yml` against `DEPLOY_URL`, failing the deploy job on a failed `must` eval; add the `http` eval kind (status, header, body and response time checks).
- TODO (agent-generated): Eval engine phase 2. Add the `trycmd` and `script` eval kinds for homer, cookbook/recipe_gen, my_lang and controlled_chaos, run against their released binaries at stage `cd`.
- TODO (agent-generated): Eval engine phase 2. Add an `eval-triage` agent skill (new requirement not met: a `FIX` naming the requirement, failing evals and evidence, unless one is open; broken or disputed eval: a `TODO`; a `cd` or `chaos` failure: rollback instructions) that opens one draft PR, and ask Jim to schedule it daily after 4:00 AM Pacific. It replaces the planned nightly FIX task above.
- TODO (agent-generated): Eval engine phase 2. Make EVL-004 (a `must` requirement with no production-stage eval) an error in `evals check`.
- TODO (agent-generated): Eval engine phase 2. Human checks for settings made by hand (Jim, 2026-11-26; see the plan's "Human checks" section): add a `standards/settings.toml` manifest of hand-configured settings (Claude MCP connectors and scheduled tasks, repo and Worker secrets, Auth0 applications, Cloudflare token scopes and Access policies, PostHog, free-plan billing) with `id`, `vendor`, `where`, `expect`, `why` and `every`, never a secret's value; add the `human-check` eval kind (`setting = "<id>"`, stage `human`) to `evals/model` and `evals check`; list due checks in `evals verdicts`; teach the `human-issues-digest` skill to add each due check to `HUMAN_ISSUES.md` as a periodic human task line (`Check <setting>: ...`) and to record Jim's "ok" or "not ok" answer as a `by = "jim"` result through `evals publish` (not ok files a `FIX`); and document the kind in `standards/docs/evals.md`.
- TODO (agent-generated): Eval engine phase 3. Tag synthetic traffic: every smoke, eval and chaos run sends the `monorepo-evals` user agent and `x-monorepo-synthetic: 1`; the PostHog snippet doesn't load for that user agent; the `telemetry` Worker marks those events `synthetic`.
- TODO (agent-generated): Eval engine phase 3. Add the `metric` eval kind with sources Web Analytics (GraphQL: LCP, INP, CLS p75, page views), Analytics Engine (errors per 1,000 views, errors after deploy, panics, CLI timings, game events), PostHog (rage, dead-click and exception session shares, funnel completion) and GitHub (deploy success rate), honouring `min_samples` and excluding synthetic events. Add `daily-observe-evals.yml` (4:00 AM Pacific) and a run one hour after each production deploy from `push-main.yml`.
- TODO (agent-generated): Eval engine phase 4. Add the `fault` eval kind and the browser fault catalogue (`offline`, `slow-3g`, `asset-500`, `auth-down`, `third-party-blocked`, `storage-denied`) as Playwright helpers, and `nightly-chaos.yml` (3:30 AM Pacific) that runs `chaos` evals only for projects whose `cd` evals passed that night, only when the repo variable `EVALS_CHAOS` is `on`, after `free-tiers.sh usage` passes, with one browser per project and a hard cap of 500 Worker requests a run.
- TODO (agent-generated): Eval engine phase 4. Add FT-006 to `free-tiers.sh check`: chaos workflows must check `EVALS_CHAOS`, run at most one job per project and never reference the RapidAPI or Auth0 management secrets. Test it in `monorepo/workflow_tests`.
- TODO (agent-generated): Eval engine phase 4. Add the signed `x-monorepo-chaos` header (HMAC with `EVALS_CHAOS_KEY`, short expiry, ignored when invalid) to the `telemetry` Worker and pane's Worker, with the `api-error`, `api-slow` and `upstream-down` faults, and ask Jim to create the secret.
- TODO (agent-generated): Eval engine phase 4. Add CI-only fault cases for CLIs (unreachable `TELEMETRY_URL`, read-only home directory, corrupt config) and controlled_chaos (missing asset, frame budget dropped).
- TODO (agent-generated): Eval engine phase 5. Read each project's `evals/feedback.jsonl` (Jim's ratings and disputes from pane, on `main`) in the verdicts, counting a "not met" rating since the last deploy.
- TODO (agent-generated): Eval engine phase 6 (in-app feedback). Add a `feedback` Rust project with a `feedback/model` crate: `Comment`, `Reply`, `Resolve` and `Anchor` types (URL and route, selector path, element text and role, offset, viewport; or scene, screen and world position and entity for games), the build info, the `feedback.jsonl` line format shared with pane's ratings, and the re-anchoring rules (selector, then text, then position; "moved" when the element is gone), all unit tested.
- TODO (agent-generated): Eval engine phase 6. Add every build's `<meta name="monorepo-build" content="commit=...;deploy=...">` tag to the web task files (Dioxus and Perseus), and bake the version and commit into the cahokia desktop and mobile apps and controlled_chaos.
- TODO (agent-generated): Eval engine phase 6. Add `feedback/toolbar`, a framework-free Rust wasm crate (`web-sys`) loaded by one `<script type="module">` tag: hidden unless `?feedback` or the hotkey turns it on, comment mode with element outline and numbered pins, kind (bug, idea, not met, looks good) and requirement picker, viewport screenshot with the pin drawn (at most 1280 px and 200 KB, WebP), last 20 console errors, sign-in through a pane popup that returns a 12-hour token by `postMessage`, sending to pane's `/api/feedback`, and showing the page's open pins and replies. Add it to cookbook/web first with Playwright tests.
- TODO (agent-generated): Eval engine phase 6. Add the feedback toolbar to every other web app and Perseus site (pane, cahokia/web, sparkhill/webapp, sparkhill/website, blueeel, bear_down, profiles/jimhillr when it deploys).
- TODO (agent-generated): Eval engine phase 6. Inject the feedback toolbar into the cahokia desktop and mobile webviews, with sign-in opened in the system browser and the token returned through the app's deep link.
- TODO (agent-generated): Eval engine phase 6. Add `feedback/bevy`, a Bevy plugin: F8 or an on-screen button pauses and enters comment mode, picks the entity under the cursor, captures a screenshot, scene, seed, frame and recent log errors, and sends through pane's `/api/feedback` (native HTTP from Rust, the toolbar's path in wasm). Add it to controlled_chaos.
- TODO (agent-generated): Eval engine phase 6. Teach `eval-triage` and the verdicts about comments: bug and not met become a `FIX` quoting the comment with its screenshot and anchor, idea becomes a `TODO`, not met and looks good count as ratings, a requirement whose evals missed a pinned bug gets a `TODO` to add one, and the fixing PR appends `reply` and `resolve` lines.
- TODO (agent-generated): Eval engine phase 7. Add a `judge-requirements` agent skill that grades each `judge` eval's rubric against production or the latest UAT screenshots, records pass or fail with a short reason and evidence through `evals publish`, and tracks agreement with Jim's ratings (rewrite TODO under 80% over the last 10). Ask Jim to schedule it weekly.
- TODO (agent-generated): Eval engine phase 8. Add `MONO-xxx` process requirements (change failure rate, lead time, time to restore, review load, eval health) computed from GitHub and the `evals-results` files in the daily observe run, and add them to the deep project review deck.

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
- pane

# On Hold Projects
- luggage
- starcraft
- profiles