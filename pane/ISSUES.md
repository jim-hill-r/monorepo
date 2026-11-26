# Priority Issues

## Review dashboard epic

Pane becomes one place for Jim to review the monorepo: deliverables and
downloads, executive reviews, human issues, a traffic-light health dashboard
with drill-down, and adding TODOs. The plan, with the architecture, health
rules and costs, is
[docs/plans/2026-11-review-dashboard.md](docs/plans/2026-11-review-dashboard.md).
Work the phases in order; within a phase, top to bottom.

### Phase 0: foundations

- TODO (agent-generated): Add a `pane/model` library crate to the pane workspace with the shared serde types (`Snapshot`, `Project`, `WorkItem`, `HumanIssue`, `Deliverable`, `Review`, `Retro`, `Signal`, `Health`) and a snapshot format version.
- TODO (agent-generated): Add a `pane/snapshot` binary crate that reads a repo checkout and writes `snapshot.json`: projects from `[monorepo].config_roots`, each project's type (its `standards/tasks/<type>.toml` include), `DEPLOY_URL`, README summary, on-hold and priority status from the root `ISSUES.md`, and every `TODO`/`FIX` line in every `ISSUES.md` with its file, line, section, epic heading and tags. Test the parsers against fixture files.
- TODO (agent-generated): Extend `pane_snapshot` to read `HUMAN_ISSUES.md` (text, added date, what it unblocks, age), `TODO`/`FIX` code comments (skipping `target/` and `node_modules/`), and the plans, decisions and retros folders (title, date, status, path).
- TODO (agent-generated): Run `pane_snapshot` in pane's `release` task before `dx bundle`, writing into the web app's static assets as `/data/snapshot.json` (never committed), and record the commit and time it was taken.
- TODO (agent-generated): Replace the placeholder pane/web app with the shell: load `/data/snapshot.json`, left nav collapsing to one column under 720 px, routes for Today, Health, Projects, Deliverables, Reviews, Human issues, Work and Add, and a projects list page. Show the snapshot's commit and age in the footer.
- TODO (agent-generated): Add Playwright `@smoke` tests for pane/web (home loads, nav works, a project page opens, no console errors), in the same shape the deploy-feedback plan sets for cookbook/web and cahokia/web, and run them in pane/web's CI.
- TODO (agent-generated): Add the retro convention to `standards/docs/documentation.md`: `standards/docs/retros/` and `<project>/docs/retros/`, files named `YYYY-MM-DD-<topic>.md`, sections What happened / What went well / What to change / Actions (checkbox list, each open action also a TODO in an `ISSUES.md`).

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

# Backlog

- TODO (agent-generated): Add a Tech radar page to pane: read `standards/radar/radar.json` into the snapshot and draw the four quadrants and four rings (Adopt, Trial, Assess, Hold) with a blip per entry, "(new)" blips marked, and a list beside it with each blip's description, exemplar projects (linking to their Project pages) and sources. See `standards/docs/tech-radar.md` for the fields.
