# Plan: pane, one place to review the monorepo

Started 2026-11-26 from Jim's request:

> It should be a single pane of glass for a human to review the work going on
> in monorepo. It should surface all the deliverables. Have downloads for apps
> or artifacts. Should render executive reviews and have a human issues
> section for actioning open issues. It should have traffic light dashboard
> with drill down for understanding the health of apps and systems based on
> observability, reviews, retros, open issues, and standards. It should have
> the ability to add a todo to the monorepo.

The work items are the issue files in [../issues/](../issues/) with
`epic: Review dashboard epic / Phase N`, one section per phase below. Each phase
ends with something Jim can open and use.

## Goal

Jim opens one site, on his laptop or phone, and within a minute knows:

1. **What needs him** right now: human issues, PRs labelled `needs-human`,
   PRs out of draft waiting for review, and new review decks.
2. **What exists**: every deliverable in the repo (live sites, releases,
   review decks, UAT reports, plans, decisions, retros), with a link or a
   download.
3. **How healthy it is**: a red, amber or green light per project and per
   shared system, and on a click, the evidence behind the colour.
4. **What to do about it**: add a `TODO` or `FIX` to any project, and mark a
   human issue done, without opening an editor or a terminal.
5. **What it earns and costs**: each project's revenue and cost by month, and
   whether it pays for itself (added 2026-11-26, see
   [Revenue and cost](#revenue-and-cost)).

Pane reads from and writes to the repo. It never becomes a second place where
work is tracked: the repo stays the source of truth (AGENTS.md), and every
change pane makes is a pull request.

## Where we start

- `pane/web` is a placeholder Dioxus 0.7 app ("This is a placeholder Dioxus
  application"). `pane/cloudflare` deploys it to the Cloudflare Pages project
  `jimhillr-pane` at <https://jimhillr-pane.pages.dev> on every push to
  `main` that touches `pane/`, and nightly.
- What pane has to surface, and where it lives today:

| Source | Where | Status 2026-11-26 |
| --- | --- | --- |
| Projects and their type | `[monorepo].config_roots` in the root `mise.toml`, the `standards/tasks/<type>.toml` each project includes, each project's `README.md` | On main |
| Production URLs | `DEPLOY_URL` in each `*/cloudflare/mise.toml` `[env]` | On main (cookbook, pane, cahokia) |
| Work items | issue files in every `issues/` folder; `TODO`/`FIX` comments in code | On main |
| Human issues | files in the root `issues/human/`; PRs labelled `needs-human` | On main (decision 0002) |
| Executive reviews | Marp decks in `docs/reviews/<date>-<project>/deck.md`, rotation table in `docs/reviews/README.md`, built to HTML and PDF by `standards/scripts/review-decks.sh` | On main; pane's release serves the decks under `/reviews/<folder>/` |
| Plans and decisions | `standards/docs/plans/`, `standards/docs/decisions/`, `<project>/docs/decisions/` | On main |
| Retros | Nowhere yet | No convention; this plan adds one |
| Standards | `mise run //:audit` (`standards/scripts/audit.sh`), `code-review` skill findings as issue files | On main |
| CI and deploys | `nightly-ci.yml`, `nightly-cd.yml`, `push-main.yml` runs; GitHub deployments to `production` | On main |
| UAT evidence | `uat-reports` Pages project, smoke tests | Planned in [deploy feedback](../../../standards/docs/plans/2026-11-deploy-feedback.md) |
| Observability | `telemetry` Worker and Analytics Engine, Cloudflare Web Analytics, PostHog replay | Planned in deploy feedback, phases 4 and 5 |
| Free-tier usage | `standards/docs/free-tiers.md`, weekly `all-free-tier-usage.yml` | [PR #582](https://github.com/jimhillr/monorepo/pull/582), open |
| Downloads | GitHub Releases for CLIs and games, review deck PDFs | Planned (CLI releases issue in the root `issues/`); decks in PR #585 |

So pane is built in the order its sources become real: repo files first,
then GitHub, then observability as the deploy-feedback plan lands it.

## Architecture

```
            GitHub Actions (push to main, nightly)
 repo files ──► pane_snapshot (Rust CLI) ──► snapshot.json ─┐
 review decks ─► Marp (review-decks.sh) ───► deck.html/pdf ─┤ static assets
                                                            ▼
 browser ◄──── pane_web (Dioxus, wasm) ◄──── Cloudflare (pane) ────────────┐
                    │                                                      │
                    └── /api/* ──► pane Worker (Rust, workers-rs) ─────────┤
                                     ├─ GitHub REST (PRs, runs, releases)  │
                                     ├─ Analytics Engine SQL (telemetry)   │
                                     └─ GitHub REST writes (add a TODO) ───┘
                                        behind Cloudflare Access
```

### Two kinds of data

- **Snapshot (from repo files).** A Rust CLI, `pane_snapshot`, walks a
  checkout of the repo and writes one `snapshot.json`: projects, types,
  `DEPLOY_URL`s, every issue file, human issues, plans, decisions,
  retros, review rotation and scorecards, audit results. It runs in pane's
  `release` task, so the JSON is built fresh into the static assets on every
  deploy and is never committed (standards/docs/mise.md: no committed build
  artifacts). Pane deploys on every push to `main` (any path, see Phase 1)
  and nightly, so the snapshot is never older than the latest merge.
- **Live (from APIs).** Things that change without a merge (open PRs, check
  runs, workflow results, releases, error counts) come from the pane Worker
  at request time, cached with the Cache API for 5 minutes (GitHub) or 15
  minutes (Analytics Engine) to stay well inside free limits.

The page shows when each piece of data was taken ("snapshot of `abc1234`,
merged 12 minutes ago"; "PRs as of 3 minutes ago").

### Crates

`pane` is already a Cargo workspace (`web`, `cloudflare`). It gains:

| Crate | Kind | Job |
| --- | --- | --- |
| `pane/model` | library | Serde types shared by everything: `Snapshot`, `Project`, `WorkItem`, `HumanIssue`, `Review`, `Signal`, `Health`. Health rules live here as plain functions so they are unit tested once and run the same in the browser. |
| `pane/snapshot` | binary | Parses the repo into a `Snapshot`. Pure parsers over file contents, tested with fixture repos. |
| `pane/web` | Dioxus web app | The UI. Reads `/data/snapshot.json` and `/api/*`. |
| `pane/cloudflare` | Worker (workers-rs) | `/api/*` routes and the static assets. Was the Pages deploy project until Phase 3; it kept its directory name (planned as `pane/worker`) so the root project list and CI didn't have to change. |

All Rust, per the language rule in `standards/docs/README.md`. Markdown is
rendered in the browser with `pulldown-cmark`.

### Hosting: Pages now, a Worker with static assets from Phase 3

Phases 0 to 2 are static and stay on the existing Pages project. Phase 3
needs server code, and Pages Functions are JavaScript, so pane moves to a
Rust Worker that serves the Dioxus build as Workers static assets and runs
only for `/api/*` (`run_worker_first = ["/api/*"]`). Static asset requests are
free and unlimited; only API calls count against the Workers free plan's
100,000 requests a day. bear_down is set to serve from Workers static assets
too, so the shared task file for that kind of project should serve both.

Done (2026-11-27): `pane/cloudflare` is the Worker `jimhillr-pane` at
<https://jimhillr-pane.jim-hill-r.workers.dev>, built from
`standards/tasks/cloudflare-worker-assets.toml`, with one route
(`/api/health`) so far. Deleting the retired Pages project `jimhillr-pane`
is a human task (`pane/docs/issues/human/delete-jimhillr-pane-pages-project-after-worker.md`).

### Access and the write path

- **Who can see it.** Everything pane shows is already public in the repo,
  but Phase 3 adds a token that can open PRs, so from then on the whole site
  sits behind **Cloudflare Access** (free up to 50 users) with a policy that
  allows only Jim's email. The Worker also checks the
  `Cf-Access-Jwt-Assertion` header on every `/api/*` call, so a
  misconfigured Access policy fails closed.
- **How writes happen.** Pane never pushes to `main`. A write (add a TODO,
  close a human issue) becomes a commit on a branch `pane/edits-<date>` and a
  pull request labelled `pane-edit`. If a `pane-edit` PR is already open,
  pane adds the commit to it instead of opening another, so a burst of quick
  notes is one PR. The existing PR triage task approves and merges it like
  any other safe human PR.
- **Which token.** A fine-grained personal access token on Jim's own
  GitHub account, limited to `jimhillr/monorepo` with Contents and Pull
  requests read and write, stored as a Worker secret. It has to be Jim's
  account, not `jimhillr-agt`: PR triage runs as `jimhillr-agt` and never
  approves its own PRs, and a pane edit is Jim's own change. The token
  expires after a year; its renewal is a human issue file.

## The site

Phone-first layout (Jim often reviews from his phone), one column under
720 px, a left nav above it.

| Page | Shows | Phase |
| --- | --- | --- |
| **Today** (home) | What needs Jim: human issues oldest first, `needs-human` PRs, ready-for-review PRs, review decks added since his last visit, and every red light. A count on each. | 1, then 3 and 4 add to it |
| **Approve** | Every green PR waiting on Jim's review, clear ones first, with its summary, attention badges and a link to its review form; stacks as one entry. Read from GitHub's public API in the browser, no token ([decision 0014](../../../standards/docs/decisions/0014-faster-pr-approvals.md)). | Built 2026-11-27, ahead of Phase 3 |
| **Acknowledge** | Every merged PR, newest first, until Jim acknowledges it after the fact, with a "Flag for follow-up" link to a prefilled issue file. Acknowledgements are per browser until Phase 5's write path. ([decision 0014](../../../standards/docs/decisions/0014-faster-pr-approvals.md)) | Built 2026-11-27, ahead of Phase 3 |
| **Health** | The traffic-light grid: one row per project and per system, one column per signal. Click a cell or row to drill down. | 4 |
| **Project** (drill-down) | One project: what it is (README summary, type, on hold or priority), its deliverables and downloads, each health signal with the evidence and links, its open TODOs and FIXes, open PRs, latest review and retro, and an "Add a TODO" button. | 1, grows each phase |
| **Deliverables** | Every deliverable by type: live sites (production and PR previews), releases with download links, review decks (HTML and PDF), UAT reports, plans, decisions, retros. | 1 to 3 |
| **Reviews** | The rotation table, and each deck rendered as slides in the page (arrow keys, full screen), with its PDF download and its PR link for feedback. | 2 |
| **Human issues** | Each open `TODO (human)` line with what it unblocks and its age, plus `needs-human` PRs. Actions: "Mark done", "Add a note", "Open on GitHub". | 1 (read), 5 (actions) |
| **Work** | Every `TODO` and `FIX` across all issue files and code comments, filterable by project, kind, priority or backlog, and tag (`agent-generated`, `agent-ignore`). | 1 |
| **Add** | A form to add a `TODO` or `FIX` to any project, or a `TODO (human)` line. | 5 |

### Deliverables and downloads

| Deliverable | Found in | Shown as |
| --- | --- | --- |
| Web apps | `DEPLOY_URL` | Link, last deploy time and commit, latest UAT report |
| PR previews | Sticky PR comment / `pr-<n>` Pages alias (deploy feedback, phase 2) | Link on the PR row |
| CLI and game binaries | GitHub Releases (CLI releases issue in the root `issues/`) | Download per OS, straight from GitHub's asset URL (no copy kept in Cloudflare) |
| Desktop and mobile builds | GitHub Releases once cahokia releases | Same |
| Review decks | `docs/reviews/*/deck.md`, built to HTML and PDF in pane's release | View in page, download PDF |
| UAT reports | `uat-reports` Pages project | Link per project, PR and nightly run |
| Plans, decisions, retros | Markdown in the repo | Rendered in page, with a link to the file on GitHub |

GitHub Actions artifacts are not offered as downloads: they expire and need a
GitHub sign-in to download. Anything worth downloading goes to a Release or is
built into pane.

### Retros (new convention)

No retros exist yet. Pane needs a fixed shape to read them, so Phase 0 adds
it to `standards/docs/documentation.md`:

- Repo-wide retros in `standards/docs/retros/`, project retros in
  `<project>/docs/retros/`, one file per retro named `YYYY-MM-DD-<topic>.md`.
- Sections: `## What happened`, `## What went well`, `## What to change`,
  and `## Actions`, a list of `- [ ]` / `- [x]` items. Each open action
  should also be an issue file in the right `issues/` folder, which the retro links.

## Health: the traffic lights

Each project and system gets a set of **signals**. Each signal is green,
amber, red, or grey (no data or not applicable), with a one-line reason and
links to its evidence. A row's light is the worst of its non-grey signals.
The row also shows coverage ("5 of 7 signals"), so a project with no
observability does not look healthier than one that has it.

The thresholds below are the starting values. They live in
`pane/health.toml`, read by `pane_snapshot` at build time, so Jim tunes them
in a one-line PR.

### Project signals

| Signal | Source | Green | Amber | Red |
| --- | --- | --- | --- | --- |
| **Build** | Latest `mise run ci` job for the project on `main` (nightly-ci or push-main) | Passed in the last 2 days | No run in 2 days | Failed |
| **Deploy and UAT** (deploying projects) | Latest production deploy and its smoke run | Deployed and smoke passed | No smoke test yet, or last deploy older than the last change | Deploy or smoke failed |
| **Observability** (production apps) | `telemetry` errors in Analytics Engine, Web Analytics | Errors flat or falling, under the daily threshold | Errors up 50% after the last deploy event | Over the red threshold, or the site is down |
| **Review** | Latest deck in `docs/reviews` and its scorecard table | Reviewed within one full rotation and no area rated Poor | One or two areas Poor, or the review is older than one rotation | Three or more areas Poor |
| **Retros** | Latest retro's `## Actions` | All actions done | An action open more than 30 days | - |
| **Open issues** | `FIX` lines and comments, human issues that block this project | No `FIX` | Any open `FIX`, or a human issue blocks it | A `FIX` open more than 14 days (by `git blame` date), or a `FIX` from a failed production smoke run |
| **Standards** | `audit.sh` findings for the project, open `code-review` findings | Audit clean | Five or more open `code-review` findings | Audit error |
| **Pull requests** | Open PRs touching the project | None older than 7 days | A PR older than 7 days, or labelled `needs-human` | A PR with failing checks older than 3 days |

"Never reviewed" and "no retro yet" are grey, not amber: absence of a review
is shown in coverage, and the review rotation already fixes it.

### System signals

| System | Source | Amber | Red |
| --- | --- | --- | --- |
| Nightly CI and CD | Latest `nightly-ci.yml`, `nightly-cd.yml` runs | Didn't run last night | Failed |
| Push to main | Latest `push-main.yml` runs | - | Latest run failed |
| Free tiers | Latest `all-free-tier-usage.yml` run (PR #582) | Over our budget | Over a vendor limit |
| PR queue | Open PRs | Oldest open more than 7 days, or more than 15 open | Any open more than 30 days |
| Human queue | `issues/human/` | Any task older than 14 days | Any task older than 30 days |
| Agent tasks | Recent PRs and commits by `jimhillr-agt` (PR triage, review rotation, digest) | No triage activity in a day | No agent activity in 3 days |
| Review rotation | `docs/reviews/README.md` | No new review in 8 days | No new review in 15 days |
| Security scans | Latest `all-codeql.yml` run, Dependabot alerts | Open moderate alert | Open high or critical alert |

### Drill-down

Clicking a light opens the project or system page scrolled to that signal.
Every signal shows: the colour and the rule that produced it, the raw numbers,
links to the evidence (the run, the deck slide, the issue file on
GitHub, the Analytics Engine chart), and the history of that signal for the
last 14 snapshots where the data allows it.

## Adding a TODO

The **Add** form asks for:

- **Where**: a project (from `config_roots`), or "Human issue" for
  `issues/human/`.
- **Kind**: `todo` or `fix`.
- **Priority**: `priority` or `backlog`, and optionally an existing epic.
- **Text**: checked against the format in
  `standards/docs/issue-management.md` before it is sent.

An issue added through pane is Jim's, so it is not tagged `agent-generated`.
A human issue gets `added` and `unblocks` filled in from the form. The
Worker writes a new issue file (one file per issue, decision 0006) in the
right `issues/` folder at the tip of `main` and commits it to the open
`pane-edit` PR (or a new one), then shows the PR link. "Mark done" on a
human issue deletes its file the same way, and "Add a note" appends a
`Note <date>: ...` paragraph to it.

Because the PR is ready, not draft, and touches only issue files, `standards/docs/pull-requests.md` should name
`pane-edit` PRs as safe for PR triage to approve. AGENTS.md's draft rule is
for PRs agents open; this one is Jim's. Phase 5 makes that change.

## Revenue and cost

Added 2026-11-26 from Jim's request:

> Pane should also include features to evaluate revenue and cost of each
> project.

Following decision [0004](../../../standards/docs/decisions/0004-database-engine.md)
(files first), the numbers come from three places, all read into
the snapshot or the Worker like everything else.

### Where the numbers come from

| Number | Source | Notes |
| --- | --- | --- |
| **Shared fixed costs** | A monthly budget per blip in `standards/radar/radar.json` (tech radar, PR #619) | Today Claude Code is $25 a month and every other blip is $0, since everything else runs on free plans. |
| **Usage against free limits** | `free-tiers.sh usage` (weekly `all-free-tier-usage.yml` run), given a JSON output uploaded as the run's artifact | Not money while we stay on free plans, so it is shown as headroom ("Pages projects 6 of 25"), not as cost. The Worker reads the latest run's JSON, cached for an hour. |
| **Project revenue and project-only costs** | A new per-project file, `<project>/finance.toml` | Revenue by month, and costs only that project has (a domain bought on Squarespace, an app store fee). Missing file means no revenue and no project-only cost. |

`finance.toml` holds whole cents in US dollars, so nothing is lost to
floating point:

```toml
# pane/finance.toml (example)
[[revenue]]
month = "2026-11"          # YYYY-MM
cents = 0
source = "None yet"        # where it came from: a store, a sponsor, an invoice

[[cost]]
name = "pane.example.com domain"
cents = 1200
every = "year"             # "month", "year" or "once"
since = "2026-11"
vendor = "Squarespace"     # must be a blip in radar.json
```

Every number has a source line, because pane shows it as evidence the same
way the health lights do.

Update 2026-11-27: revenue and project-only costs live in each project's
`docs/cost.toml` ([standards/docs/cost.md](../../../standards/docs/cost.md)),
which replaces `finance.toml` with the same fields. The rules below are the
`money` module in [`pane/model`](../../model/README.md#money).

### Splitting shared costs

A fixed cost such as Claude Code isn't any one project's. Pane splits each
month's shared costs across projects **by their share of the PRs merged that
month** (a PR touching two projects counts half to each), since agent work is
what the money buys. Projects with no merged PRs that month get none. The
rule is one pure function in `pane/model`, so it is unit tested and easy to
change; the page always shows the split and the rule beside it.

### What pane shows

- **Money page**: one row per project with this month's revenue, direct cost,
  share of shared costs and net, and a 12-month chart of each. A totals row,
  and the shared costs listed on their own with their radar blips.
- **Project page**: a Money section with the same numbers for that project,
  each linked to its `finance.toml` line or radar blip on GitHub.
- **Free tiers**: the headroom table, so a limit coming close shows up before
  it would cost anything.
- **Add**: "Add revenue" and "Add a cost" forms that write a `finance.toml`
  entry through the `pane-edit` PR, the same way TODOs are added.

No new vendor or service: the numbers are files in the repo and one more
artifact from a workflow that already runs. Pane never reads a bank,
payment processor or billing API; if one is wanted later it needs its own
decision record.

## Phases

### Phase 0: foundations

- `pane/model` and `pane/snapshot` crates. The snapshot covers projects,
  types, `DEPLOY_URL`s, READMEs and every issue file, with parsers
  tested against fixture files.
- The web app loads `snapshot.json`, has the nav and the phone-first layout,
  and shows the projects list.
- pane's `release` runs `pane_snapshot` before `dx bundle`.
- Playwright `@smoke` tests for pane/web, in the same shape the deploy
  feedback plan sets for cookbook and cahokia.
- The retro convention in `standards/docs/documentation.md`.

### Phase 1: read-only from the repo

- Today, Work, Human issues (read only), Project pages, and Deliverables for
  live sites, plans, decisions and retros, with Markdown rendered in page.
- Pane deploys on every push to `main`, not only pushes under `pane/`, since
  its snapshot reads the whole repo. Direct uploads use no Pages builds, so
  this costs nothing.

### Phase 2: executive reviews

PR #585 is merged.

- pane's release runs `standards/scripts/review-decks.sh` and copies the
  HTML and PDF decks into the static assets under `/reviews/`.
- The Reviews page: rotation table, deck viewer, PDF download, link to the
  review PR for comments. Built: `/reviews` and `/review/<folder>`, with
  review decks on Deliverables and the latest review on each Project page.
- The snapshot parses each deck's Scorecard slide (`| Area | Rating | Why |`)
  so Phase 4 can use it. The `deep-project-review` skill should keep that
  table's shape; Phase 2 adds a line saying so to the skill. Built: each
  review's `scorecard` in the snapshot, shown as a table on the review's
  page and as counts on the Reviews and Project pages.

### Phase 3: live GitHub data

- Move pane from Pages to a Rust Worker with static assets, put Cloudflare
  Access in front of it, and retire the `pane` Pages project.
- `/api/github/*` routes, cached for 5 minutes: open PRs (draft, ready,
  `needs-human`, age, checks), latest workflow runs per project and per
  system, releases and their assets, production deployments.
- Today gains PRs; Deliverables gains releases (downloads) and previews.

### Phase 4: the traffic-light dashboard

- `pane/health.toml` with the thresholds above, and the health rules in
  `pane/model`, unit tested signal by signal.
- The Health page and the drill-down sections on project and system pages.
- The `audit.sh` findings per project in the snapshot: the audit runs in
  pane's release and `pane_model::audit` parses its output (done, order
  152).
- Signal history: the Worker keeps the last 14 daily health results in KV
  (one write a day, far below the free 1,000).

### Phase 5: actions

- The Add form and the "Mark done" and "Add a note" actions, through the
  `/api/edits` route and the `pane-edit` PR.
- The PR triage rule for `pane-edit` PRs in `standards/docs/pull-requests.md`.

### Phase 6: observability

Needs the `telemetry` Worker and Web Analytics from the deploy-feedback plan
(root `issues/` backlog).

- `/api/telemetry/*`: error counts per app and build from the Analytics
  Engine SQL API, cached 15 minutes, and deploy events to line them up with.
- The Observability signal and its chart on each app page; links to the
  app's Web Analytics and to PostHog recordings once those exist.
- Link the latest UAT report per project once `uat-reports` exists.

### Phase 7: revenue and cost

Needs Phase 1 for the pages, Phase 3 for the free-tier usage, and Phase 5 for
the forms. Can start after Phase 1.

- The `finance.toml` convention in `standards/docs/`, money types and the
  shared-cost split in `pane/model`, and the snapshot reading every
  `finance.toml` and the radar's monthly budgets.
- The Money page and the Money section on Project pages.
- `free-tiers.sh usage` writes JSON, and the Worker serves the latest run's
  headroom.
- The "Add revenue" and "Add a cost" forms through the `pane-edit` PR.

## Things Jim does

These become human issue files in `issues/human/` in the PR of the phase
that needs them, not before, so the digest only shows what is actionable:

- Phase 3: create the fine-grained token (Contents and Pull requests write on
  `jimhillr/monorepo`, one-year expiry) and add it as the Worker secret; set
  up the Cloudflare Access application and its policy for the pane hostname;
  give the Cloudflare deploy token permission to deploy Workers if it lacks
  it.
- Phase 6: the Analytics Engine read permission already asked for in
  `issues/human/`.

## Cost

All on approved vendors (GitHub, Cloudflare) and free plans. Numbers from
`standards/docs/free-tiers.md` (PR #582).

| Piece | Free limit | Pane's use |
| --- | --- | --- |
| Pages project, then Worker static assets | 100 projects; static asset requests unlimited | 1 project, then 1 Worker (budget 25) |
| Worker requests (`/api/*` only) | 100,000 a day | A few hundred a day for one viewer |
| GitHub REST API with a token | 5,000 requests an hour | Cached 5 minutes: under 100 an hour |
| Analytics Engine queries | 10,000 a day | Cached 15 minutes: under 200 a day |
| KV | 1,000 writes a day | 1 write a day |
| Cloudflare Access | 50 users | 1 |
| Cron Triggers | 5 per account (budget 3) | None: the snapshot is built by GitHub Actions |
| GitHub Actions | Free on a public repo | One pane build per push to `main`, plus Marp builds |

Pane adds no vendor and nothing that bills. If any of these limits is hit,
the page shows the snapshot with a "live data unavailable" banner rather than
retrying.

## Decisions for Jim

Approving the PR that adds this plan approves these. Say if any should change.

1. Pane moves from Pages to a Rust Worker with static assets in Phase 3, to
   keep server code in Rust.
2. Pane sits behind Cloudflare Access from Phase 3, allowing only Jim.
3. Writes go through a fine-grained token on Jim's own GitHub account, as
   ready-for-review `pane-edit` PRs that PR triage may approve.
4. Retros follow the convention above.
5. Thresholds start as in this plan and are tuned in `pane/health.toml`.
6. Revenue and project-only costs live in a per-project `finance.toml`, in
   whole US cents, and shared costs come from the radar's monthly budgets.
   Revenue added through pane goes through a `pane-edit` PR like a TODO.
7. Shared costs are split across projects by their share of merged PRs that
   month.

## Not doing (yet)

- Approving or merging PRs from pane. GitHub already does that well, and
  pane links there.
- Notifications. The weekly human-issues digest already covers that.
- Editing TODOs in place or reordering them. Pane adds lines and removes
  human issues; anything bigger is a normal PR.
- Watching Claude scheduled tasks directly: pane sees them only through the
  PRs and commits they make.
