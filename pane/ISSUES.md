# Priority Issues

## Review dashboard epic

Pane becomes one place for Jim to review the monorepo: deliverables and
downloads, executive reviews, human issues, a traffic-light health dashboard
with drill-down, and adding TODOs. The plan, with the architecture, health
rules and costs, is
[docs/plans/2026-11-review-dashboard.md](docs/plans/2026-11-review-dashboard.md).
Work the phases in order; within a phase, top to bottom.

### Phase 1: read-only from the repo

- TODO (agent-generated): Build the Work page: every `TODO` and `FIX` across the repo, filterable by project, kind, priority or backlog, and tag, each linking to its line on GitHub.
- TODO (agent-generated): Build the Human issues page (read only): each open human issue line with what it unblocks and its age, oldest first, linking to the line on GitHub.
- TODO (agent-generated): Build the Project page: README summary, type, status, deliverables, open TODOs and FIXes, latest plan, decision and retro links.
- TODO (agent-generated): Build the Deliverables page for live sites (`DEPLOY_URL`), plans, decisions and retros, rendering Markdown in page with `pulldown-cmark` and linking each file on GitHub.
- TODO (agent-generated): Build the Today page from the snapshot: open human issues oldest first and items added since the last visit (last visit kept in `localStorage`, page works without it).
- TODO (agent-generated): Deploy pane on every push to `main`, not only pushes under `pane/`, since its snapshot reads the whole repo. Do it in `push-main.yml` or `mise-projects.yml` without changing how other projects are picked.

### Phase 2: executive reviews

Needs PR #585 (docs/reviews and `review-decks.sh`) merged.

- TODO (agent-generated): In pane's release, run `standards/scripts/review-decks.sh` and copy each deck's HTML, PDF and images into the static assets under `/reviews/<folder>/`.
- TODO (agent-generated): Build the Reviews page: the rotation table from `docs/reviews/README.md`, each deck shown as slides in page (keyboard and full screen), its PDF download, and its review PR link for comments. Add review decks to Deliverables and the latest review to each Project page.
- TODO (agent-generated): Parse each deck's Scorecard slide (`| Area | Rating | Why |`) into the snapshot, and add a line to the `deep-project-review` skill saying pane reads that table so its shape must stay.

### Phase 3: live GitHub data

- TODO (agent-generated): Move pane from the Pages project to a Rust Worker (workers-rs) that serves the Dioxus build as Workers static assets with `run_worker_first` for `/api/*` only. Add or extend a shared task file for Workers-with-assets projects (bear_down needs the same), update `DEPLOY_URL`, and add a `HUMAN_ISSUES.md` line to delete the old `pane` Pages project once the Worker is live.
- TODO (agent-generated): Put pane behind Cloudflare Access allowing only Jim, and make every `/api/*` route verify the `Cf-Access-Jwt-Assertion` JWT against the Access team's keys and fail closed. Add the `HUMAN_ISSUES.md` lines for creating the Access application and policy, and for any deploy-token permission the Worker deploy needs.
- TODO (agent-generated): Add `/api/github/*` routes to the pane Worker, cached for 5 minutes with the Cache API: open PRs (draft, ready, `needs-human`, age, checks, projects touched), latest workflow runs per project and per system workflow, releases with asset download URLs, and production deployments. Add the `HUMAN_ISSUES.md` line for the GitHub token secret (read scope is enough until Phase 5).
- TODO (agent-generated): Show live data in the app: PRs on Today and Project pages, releases as downloads and PR previews on Deliverables, a "live data unavailable" banner when the API fails, and the age of each live piece.

### Phase 4: traffic-light dashboard

- TODO (agent-generated): Add `pane/health.toml` with the starting thresholds from the plan, and the health rules in `pane/model` as pure functions (one per signal, worst-of roll-up, coverage count), with unit tests for every threshold edge.
- TODO (agent-generated): Compute the project signals (Build, Deploy and UAT, Review, Retros, Open issues, Standards, Pull requests) from the snapshot and live data. Add per-project `audit.sh` findings to the snapshot (parse its output, or give the audit a machine-readable mode) and `git blame` ages for `FIX` lines.
- TODO (agent-generated): Compute the system signals (nightly CI and CD, push to main, free tiers, PR queue, human queue, agent tasks, review rotation, security scans).
- TODO (agent-generated): Build the Health page (grid of projects and systems by signal, worst-of light per row, coverage) and the drill-down sections on Project and system pages with each signal's rule, numbers and evidence links. Add red lights to Today.
- TODO (agent-generated): Keep the last 14 daily health results in Workers KV (one write a day) and show each signal's history on the drill-down.

### Phase 5: actions

- TODO (agent-generated): Add an `/api/edits` route to the pane Worker that inserts a line into a chosen `ISSUES.md` section or `HUMAN_ISSUES.md` at the tip of `main` and commits it to the open `pane-edit` PR, or opens a new ready-for-review PR labelled `pane-edit` from a `pane/edits-<date>` branch. Validate the line against `standards/docs/issue-management.md`. Update the `HUMAN_ISSUES.md` token line to Contents and Pull requests write on Jim's own account.
- TODO (agent-generated): Build the Add page (project or human issue, TODO or FIX, section, text) and the "Mark done" and "Add a note" actions on human issues, each showing the resulting PR link.
- TODO (agent-generated): Name `pane-edit` PRs that touch only `ISSUES.md` or `HUMAN_ISSUES.md` as safe for PR triage to approve in `standards/docs/pull-requests.md`, and note in AGENTS.md that they are Jim's PRs and open ready, not draft.

### Phase 6: observability

Needs the `telemetry` Worker and Web Analytics from the deploy-feedback TODOs in the root `ISSUES.md`.

- TODO (agent-generated): Add `/api/telemetry/*` routes that query the Analytics Engine SQL API for error counts per app and build and for deploy events, cached for 15 minutes.
- TODO (agent-generated): Compute the Observability signal for each production app and show its error chart with deploy markers on the app's page, with links to its Web Analytics and PostHog recordings once those exist.
- TODO (agent-generated): Link each project's latest UAT report from the `uat-reports` Pages project on its Project page and in Deliverables, and use the smoke result in the Deploy and UAT signal.

### Eval engine: requirements and feedback

Needs Phases 3 to 5 above and the eval engine's `evals-results` files (root
`ISSUES.md`, eval engine phase 1). The plan is
[../standards/docs/plans/2026-11-eval-engine.md](../standards/docs/plans/2026-11-eval-engine.md).

- TODO (agent-generated): Add a Requirements view to each Project page: each requirement's verdict (from `evals/model`), each eval's latest result with its evidence link, and its history, read into the snapshot at build time from the `evals-results` branch (`verdicts.json` and `results/`), and redeploy pane after each eval run. Add a Requirements signal (worst `must` verdict, with requirement coverage) to the Health page.
- TODO (agent-generated): Add Rate (met, not met, not sure, with a note), Dispute an eval and Propose a requirement actions, written through the `pane-edit` PR as JSON lines in `<project>/evals/feedback.jsonl` (and a draft entry in `requirements.md` for proposals), and name `evals/feedback.jsonl` as safe for PR triage to approve alongside `ISSUES.md` in `standards/docs/pull-requests.md`.
- TODO (agent-generated): List requirements that Jim has not rated since their last change or deploy on the Today page, a few at a time, and show judge agreement per rubric once judge evals exist.
- TODO (agent-generated): For in-app feedback (eval engine phase 6), add `/feedback/sign-in` (behind Access; returns a signed 12-hour token to the opener by `postMessage`, or to a deep link for desktop and mobile), `POST /api/feedback` (checks the token, validates the comment with `feedback/model`, writes the `feedback.jsonl` line and the screenshot into the open `pane-edit` PR) and `GET /api/feedback?project=&url=` (open pins for a page from the snapshot plus the open `pane-edit` PR), with CORS limited to the projects' `DEPLOY_URL`s and preview hosts. Name `evals/feedback/*.webp` as safe for PR triage alongside `feedback.jsonl`.
- TODO (agent-generated): Add a Feedback page: every comment, filterable by project, kind, status and requirement, with its screenshot and pin, build, replies, and an "Open in app" link that loads the URL with the toolbar on and the pin highlighted. Show open pins on each Project page.

### Phase 7: revenue and cost

Needs Phase 1 for the pages, Phase 3 for free-tier usage and Phase 5 for the forms. See "Revenue and cost" in the plan. Revenue and project-only costs live in each project's `docs/cost.toml` (`[[revenue]]` and `[[cost]]`, convention in `standards/docs/cost.md`), which replaces the `finance.toml` the plan proposed; read them through the `cost_model` crate (`cost/model`), which already has the money types and `monthly_cents`.

- TODO (agent-generated): Add a `ProjectMoney` type to `pane/model` built on `cost_model`'s `Cost`, `Revenue` and `monthly_cents`, and pure functions for a project's monthly revenue, direct cost and net, and for splitting shared costs across projects by their share of PRs merged that month, with unit tests for each edge (no PRs, one project, a PR touching two projects, a once cost).
- TODO (agent-generated): Read every `docs/cost.toml` (through `cost_model::parse_cost_file`), the radar's monthly budgets (`standards/radar/radar.json`) and the limits in `standards/radar/limits.toml` into the snapshot in `pane_snapshot`: per project its services, plans, money this month and expected usage, and account-wide expected usage against each budget. Fail the build on an invalid file, with fixture tests.
- TODO (agent-generated): Build the Money page (per project: this month's revenue, direct cost, shared-cost share and net, a 12-month chart, a totals row, and shared costs listed with their radar blips) and a Money section on each Project page, each number linking to its source line on GitHub.
- TODO (agent-generated): Give `free-tiers.sh usage` a JSON output uploaded as the `all-free-tier-usage.yml` run's artifact, and add `/api/free-tiers` to the pane Worker (latest run's JSON, cached for an hour) shown as a headroom table on the Money page.
- TODO (agent-generated): Add "Add revenue" and "Add a cost" forms that append a `[[revenue]]` or `[[cost]]` entry to the project's `docs/cost.toml` through the `pane-edit` PR (which `cost check` then verifies), and name a `docs/cost.toml` change as safe for PR triage to approve in `standards/docs/pull-requests.md` only when the PR is a `pane-edit` PR that adds revenue or a $0 cost; any cost above $0 stays `needs-human`.

# Backlog

- TODO (agent-generated): Add a Tech radar page to pane: read `standards/radar/radar.json` into the snapshot and draw the four quadrants and four rings (Adopt, Trial, Assess, Hold) with a blip per entry, "(new)" blips marked, and a list beside it with each blip's description, exemplar projects (linking to their Project pages) and sources. See `standards/docs/tech-radar.md` for the fields.
