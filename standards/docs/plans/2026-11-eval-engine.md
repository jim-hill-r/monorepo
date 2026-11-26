# Plan: an eval engine for product requirements

Started 2026-11-26 from Jim's request:

> I'd like to build a robust eval engine into our processes. After ci, after
> cd, we have observability we can eval against, human feedback as a part of
> pane, and automated in production chaos testing to evaluate product
> requirements against as implemented.

The decision behind this plan is
[../decisions/0003-eval-engine.md](../decisions/0003-eval-engine.md). The work
items are `TODO (agent-generated)` lines in the root
[ISSUES.md](../../../ISSUES.md) under `# Backlog` ("Eval engine"), and in
[pane/ISSUES.md](../../../pane/ISSUES.md) for the pane pages, one group per
phase below.

## Goal

For every project, at any time, answer one question with evidence: **does
what we shipped do what we said it should?**

- Each project writes down its **product requirements**, each with an ID.
- Each requirement has one or more **evals**: checks that say pass or fail
  (or give a number against a threshold) for that requirement.
- Evals run at every point Jim named: **after CI**, **after CD**, against
  **observability** data, as **human feedback** in pane, and as **chaos
  tests in production**. An agent **judge** covers what can't be automated.
- Every run writes **results** to one place, so each requirement gets a
  verdict (met, at risk, not met, unknown) with a link to the evidence, and a
  history.
- A failing requirement becomes a `FIX` in the project's `ISSUES.md`, so the
  existing agent loop works it. Nothing new to watch.

Tests already tell us the code does what the code says. Evals tell us the
product does what the requirement says, in production, for real users and
under failure. Most evals reuse tests we already have or plan (Playwright
`@smoke`, `trycmd`); the engine adds the link to a requirement, the
production stages, and the verdict.

## Where we start

| Piece | State on 2026-11-26 |
| --- | --- |
| Product requirements | None written down. Intent lives in READMEs, `ISSUES.md` epics, plans and the review decks' Goals slide. |
| CI | `mise run ci` per project on every PR (`pull-request-ci.yml`), on push to main, and nightly at 2 AM Pacific (`nightly-ci.yml`). |
| CD | `push-main.yml` deploys affected projects; `nightly-cd.yml` redeploys the latest green main of every deploying project at 2:30 AM Pacific (PR #583, #595). `DEPLOY_URL` per project. |
| UAT | Planned: Playwright `@smoke` against previews and production, `trycmd` for CLIs, scripted Bevy runs ([deploy feedback](2026-11-deploy-feedback.md), phases 2 and 3). |
| Observability | Planned: `telemetry` Worker with Analytics Engine, Cloudflare Web Analytics, PostHog replay (deploy feedback, phases 4 and 5; decision 0001). |
| Human review | Planned: pane ([plan](../../../pane/docs/plans/2026-11-review-dashboard.md), PR #606), with a traffic-light Health page and `pane-edit` PRs for writes. |
| Chaos testing | None. |
| Free tiers | Limits, budgets and guards in [free-tiers.md](../free-tiers.md) (PR #582). |

So the engine is built in the order its inputs become real: requirements and
CI first, then production stages as the deploy-feedback plan lands them, then
pane.

## Concepts

```
requirements.md            evals.toml                 results (D1)          verdicts
COOK-001 "readable   ──►  recipe-mobile (ci, cd)  ──► pass @ abc123 cd  ──► COOK-001: Met
 on a phone" (must)       recipe-lcp (observe)        2310 ms ≤ 2500         (5 evals, 2 h old)
                          recipe-offline (chaos)      pass                       │
                          Jim's rating (human)        "met" 2026-12-02          ▼
                                                                         pane Health, FIX lines
```

### Requirements

Each project that ships something gets `<project>/docs/requirements.md`. One
`##` heading per requirement, with a stable ID made of a short project prefix
and a number, and three fixed fields:

```markdown
# Cookbook requirements

## COOK-001: A recipe is easy to cook from on a phone

- Priority: must
- Why: Jim cooks from his phone, one hand free.
- Accept: no sideways scrolling at 375 px wide; body text at least 16 px;
  the recipe is readable within 2.5 s on a 4G phone (LCP p75).
```

- **Priority** is `must`, `should` or `could`. Only `must` requirements can
  turn a project red.
- **Accept** says what "met" means in words a person can check. Evals are
  written from it.
- IDs are never reused. A dropped requirement stays with
  `- Status: retired (<date>, <why>)`, so old results still resolve.
- The prefix is listed once in [naming.md](../naming.md) per project
  (`COOK`, `PANE`, `CAHO`, `HOME`, ...).

Requirements are Jim's product intent, so a PR that adds or changes one is
labelled `needs-human` and waits for his approval. An agent may draft them
(from the README, the review deck's Goals slide, plans and epics), but never
merge them.

### Evals

Each project lists its evals in `<project>/evals.toml`, next to its
`mise.toml`:

```toml
prefix = "COOK"

[[eval]]
id = "recipe-mobile-layout"
requirement = "COOK-001"
stages = ["ci", "cd"]
kind = "playwright"
grep = "@req:COOK-001"

[[eval]]
id = "recipe-lcp"
requirement = "COOK-001"
stages = ["observe"]
kind = "metric"
source = "web-analytics"
metric = "lcp_p75_ms"
max = 2500
window = "7d"

[[eval]]
id = "recipe-offline-after-load"
requirement = "COOK-004"
stages = ["chaos"]
kind = "fault"
fault = "offline"
grep = "@req:COOK-004"

[[eval]]
id = "recipe-voice"
requirement = "COOK-007"
stages = ["judge"]
kind = "judge"
rubric = "Reads like a friendly home cook, not a food magazine. Fail if any intro is over 80 words."
```

Eval kinds:

| Kind | What runs | How it maps to a requirement |
| --- | --- | --- |
| `cargo` | `cargo test` | Test name starts with the requirement ID, `cook_001_...` |
| `playwright` | Playwright, JSON reporter | `@req:COOK-001` tag in the test title (alongside `@smoke`) |
| `trycmd` | CLI transcript tests | Case file under `tests/cmd/COOK-001-*.toml` |
| `script` | Bevy scripted run (deploy feedback, phase 3c) | Named check in the script file |
| `http` | One request to a URL with expected status, header or body | Declared inline in `evals.toml` |
| `metric` | A query against observability data with a threshold | Declared inline: `source`, `metric`, `min`/`max`, `window` |
| `fault` | A Playwright test run with a fault injected | `fault` name from the catalogue below plus a `@req:` tag |
| `judge` | An agent grades the live product against a rubric | `rubric` inline; see Judge evals |
| `human` | Jim's rating in pane | Every requirement gets this implicitly; no entry needed |

`mise run //:audit` checks the files (Phase 0): every eval names a
requirement that exists and isn't retired, every ID is unique, every
`playwright`/`cargo`/`trycmd` eval matches at least one real test, and every
`must` requirement has at least one eval that runs in production (`cd`,
`observe` or `chaos`). A `must` with no production eval is a warning at
first and becomes an error in Phase 4.

### Stages

| Stage | When | Runs against | Gates |
| --- | --- | --- | --- |
| `ci` | Every PR, push to main, nightly CI (2:00 AM Pacific) | The build in the runner; the PR preview once previews exist | The PR: a failing `must` eval fails the check, like any test |
| `cd` | Right after each production deploy (push to main and nightly CD, 2:30 AM) | `DEPLOY_URL`, the released binary | Nothing automatically (see Decision 4); a failure files a `FIX` and offers a rollback |
| `chaos` | Nightly, 3:30 AM Pacific, only after that night's `cd` evals passed | Production, with faults injected (see below) | Nothing; files a `FIX` |
| `observe` | Daily at 4:00 AM Pacific over the trailing window, and 1 hour after each deploy | Analytics Engine, Web Analytics, PostHog, GitHub | Nothing; files a `FIX` |
| `judge` | Weekly, Claude scheduled task | Production plus the latest UAT report | Advisory only |
| `human` | Whenever Jim rates in pane | His own use | Overrides `judge`; see Verdicts |

All schedules are GitHub Actions `schedule` triggers, so none uses one of the
5 Cloudflare Cron Triggers.

### Results

Every eval run produces one record per eval:

| Field | Example |
| --- | --- |
| `project`, `requirement`, `eval` | `cookbook`, `COOK-001`, `recipe-lcp` |
| `stage`, `kind` | `observe`, `metric` |
| `commit`, `deploy` | `abc1234`, the GitHub deployment id (for `cd`, `chaos`, `observe`) |
| `status` | `pass`, `fail`, `error` (the eval itself broke), `skip` |
| `value`, `threshold` | `2310`, `<= 2500` |
| `evidence` | Link to the run, UAT report, PostHog recording or Analytics Engine query |
| `run`, `at` | Workflow run id, UTC time |
| `by` | `ci`, `agent` (judge) or `jim` (human) |

Records go three places:

1. **The job summary**, as a table per project: requirement, eval, status,
   evidence. This works from Phase 0 with nothing else set up.
2. **A results file**, `evals-results.jsonl`, uploaded as a 14-day Actions
   artifact for debugging.
3. **Cloudflare D1**, database `evals`, table `results`, written with
   `wrangler d1 execute --remote` at the end of each run. This is the
   history pane and the triage skill read. At our scale (about 20 projects,
   a few hundred evals, a handful of runs a day) that is under 5,000 rows a
   day against D1's free 100,000 writes. A weekly step deletes raw rows
   older than 180 days after rolling them up into a `daily` table (one row
   per eval per day).

Human feedback is the exception: it is Jim's own judgment, so it lives in the
repo (see Human feedback) and the nightly run copies it into D1.

### Verdicts

Each requirement gets one verdict, computed by the same Rust function in the
engine and in pane:

| Verdict | Rule |
| --- | --- |
| **Not met** (red) | The latest result of any eval in `cd`, `chaos` or `observe` failed, or Jim's latest rating since the last deploy is "not met". For a `should` or `could` requirement this shows as amber. |
| **At risk** (amber) | An eval failed 2 or more of its last 7 runs (flaky), a `metric` is within 10% of its threshold, the judge says not met, a `ci` eval is failing on main, or the newest production result is older than 3 days. |
| **Met** (green) | Every eval's latest result passes, and at least one production-stage result is under 2 days old. |
| **Unknown** (grey) | No evals, or no production-stage results yet. |

`error` results count as at risk, not as failures of the product: the fix is
in the eval. A project's requirement coverage ("9 of 12 musts evaluated in
production") is shown beside its verdicts, so a project with few evals never
looks healthier than one with many, the same rule pane uses for signals.

Thresholds (10%, 2 of 7, 2 and 3 days) live in `evals/verdicts.toml` and
are tuned in a one-line PR.

## The engine

A new Rust project, `evals` (type `rust`, in `config_roots`), with two
crates:

| Crate | Job |
| --- | --- |
| `evals/model` | Serde types (`Requirement`, `Eval`, `EvalResult`, `Verdict`), the parsers for `requirements.md` and `evals.toml`, and the verdict rules. No I/O. Pane depends on it for its Requirements signal, so the rules run the same in both. |
| `evals/cli` | The `evals` binary. |

Commands:

- `evals check` — validate every project's requirements and evals (run by
  `//:audit`).
- `evals run --stage <stage> --project <p>` — run or collect the evals for
  one stage. For `cargo`, `playwright`, `trycmd` and `script` it does not run
  the tests itself: it reads the reports the project's own tasks already
  wrote (Cargo's JSON test output, Playwright's JSON reporter, trycmd's
  output), so tests run once. For `http`, `metric` and `fault` it does the
  work.
- `evals publish` — write the job summary, the JSONL artifact and the D1
  rows.
- `evals verdicts [--project <p>]` — print the current verdicts from D1
  (used by the triage skill and by people).

Each shared task file in `standards/tasks/` gains an `evals` task
(`evals run --stage $EVALS_STAGE --project $PROJECT`), so the workflows call
it the same way they call `ci` and `deploy`. `mise-projects.yml` runs it
after `ci` (stage `ci`) and after `deploy` and `smoke` (stage `cd`).

New workflows:

| Workflow | Schedule | Job |
| --- | --- | --- |
| `nightly-chaos.yml` | 3:30 AM Pacific | `chaos` evals for every deploying project whose `cd` evals passed tonight |
| `daily-observe-evals.yml` | 4:00 AM Pacific, and called by `push-main.yml` with a 1-hour delay job | `observe` evals |

## Stage details

### After CI

- Tag existing tests with requirement IDs as requirements are written:
  `@req:` in Playwright titles, ID-prefixed Rust test names, ID-prefixed
  `trycmd` cases.
- `evals run --stage ci` after `mise run ci`, in PR CI and nightly CI.
- The sticky PR comment (deploy feedback, phase 2) gains a short list: the
  requirements whose evals this PR touched, and their status on the preview.
  That tells a reviewer which product promises the change could affect.

### After CD

- `cd` evals run against `DEPLOY_URL` (web apps) or the released binary
  (CLIs, games) right after `deploy` and `smoke`. Most are the same
  Playwright tests as `ci`, pointed at production with `BASE_URL`.
- `http` evals cover what a browser test doesn't need to: security headers,
  redirects, a `404` page, response time of one request.
- A failing `must` eval fails the deploy job (the same way a failed smoke
  does in the deploy-feedback plan), and the nightly triage files a `FIX`
  and offers the rollback `push-main.yml` already supports.

### Against observability

`metric` evals query the sources the deploy-feedback plan adds. Each is a
named metric the engine knows how to compute:

| Source | Metrics | API (free) |
| --- | --- | --- |
| Cloudflare Web Analytics | `lcp_p75_ms`, `inp_p75_ms`, `cls_p75`, `page_views` | GraphQL Analytics API |
| `telemetry` Worker (Analytics Engine) | `errors_per_1k_views`, `errors_after_deploy` (ratio to the 7 days before), `panics`, game `level_reached`, CLI `command_p95_ms` | Analytics Engine SQL API (10,000 queries a day) |
| PostHog | `rage_click_sessions_pct`, `dead_click_sessions_pct`, `exception_sessions_pct`, a named funnel's completion | PostHog query API (read-only key) |
| GitHub | `deploy_success_rate`, `time_since_last_deploy` | REST, `GITHUB_TOKEN` |

Rules:

- **Low traffic is not a pass.** If a window has fewer samples than the
  eval's `min_samples` (default 30), the result is `skip` and the verdict
  falls back to the other stages. Most of our apps have little traffic, so
  production chaos and `cd` evals carry most of the weight for now.
- **Our own traffic is excluded.** Every eval and smoke run sends a
  `monorepo-evals` user agent and an `x-monorepo-synthetic: 1` header. The
  PostHog snippet doesn't load for that user agent (so no recording is
  spent), the `telemetry` Worker tags those events `synthetic` and metric
  queries filter them out, and Web Analytics ignores headless browsers
  already. Without this, chaos runs would show up as real users' errors.
- **Deploy-relative checks.** `errors_after_deploy` compares the hour after
  each `deploy` event with the week before it, which is how a regression
  shows up when traffic is low.

### Human feedback in pane

Pane gains a **Requirements** view per project (in its Project page) and a
**Requirements** signal on the Health page. For each requirement it shows the
verdict, each eval's latest result with evidence, and the history.

Jim can act on each requirement:

- **Rate it**: Met, Not met, or Not sure, with an optional note. The rating
  applies to the current production deploy.
- **Dispute an eval**: "this eval is wrong" (flaky, testing the wrong thing,
  bad threshold), with a note.
- **Propose a requirement**: a new requirement for the project, which opens
  as a draft entry in `requirements.md`.

Writes go through pane's existing write path, the `pane-edit` PR, so the repo
stays the source of truth. Ratings and disputes are appended as JSON lines to
`<project>/evals/feedback.jsonl`:

```json
{"at":"2026-12-02T18:04:00Z","by":"jim","requirement":"COOK-001","deploy":"123456","rating":"not_met","note":"Header overflows on my phone"}
{"at":"2026-12-02T18:05:00Z","by":"jim","eval":"recipe-lcp","dispute":"threshold too strict for a cold cache"}
```

The nightly run loads these into D1. A "not met" rating files a `FIX` with
Jim's note; a dispute files a `TODO (agent-generated)` to fix or remove the
eval. Pane's Today page lists requirements that have never been rated by
Jim and changed since his last rating, a few at a time, so feedback is a
two-minute habit rather than a chore.

### Chaos testing in production

The goal is to check requirements under failure ("a recipe stays readable if
the network drops", "pane shows the snapshot when GitHub is down") against
the real production build, without hurting anyone, without touching any
third party's systems, and without spending free-tier room we need.

**How faults are injected.** Only in two places we own:

1. **In the browser.** A Playwright run against production, where the test
   itself breaks things for its own session only, using `page.route`, CDP
   network conditions and browser context options. This needs nothing
   deployed and touches no shared infrastructure.
2. **In our own Workers** (pane's Worker, `telemetry`, bear_down when they
   exist). A request carrying `x-monorepo-chaos: <fault>` and a valid HMAC
   signature (secret `EVALS_CHAOS_KEY`, short expiry) makes that one request
   fail in the named way. Without a valid signature the header is ignored.
   Other users' requests are never affected.

**Fault catalogue** (Phase 4 starts with the first six):

| Fault | Injected by | Example requirement it checks |
| --- | --- | --- |
| `offline` | Browser goes offline after first load | Recipe stays readable; app shows an offline notice, not a blank page |
| `slow-3g` | CDP network throttling, 4x CPU slowdown | Readable within the requirement's time on a slow phone |
| `asset-500` | `page.route` returns 500 for one wasm/JS/CSS asset | A friendly error with a reload, not a white screen |
| `auth-down` | `page.route` blocks the Auth0 domain | Signed-out features still work; sign-in shows a clear error. Never signs in for real, so no Auth0 quota is used |
| `third-party-blocked` | Block PostHog, Web Analytics, `telemetry` | The app works the same with trackers blocked |
| `storage-denied` | Context with storage disabled or cleared mid-session | No crash; preferences fall back to defaults |
| `api-error`, `api-slow` | Signed chaos header on our Worker | Pane shows "live data unavailable" and the snapshot (pane plan) |
| `upstream-down` | Signed header makes our Worker act as if GitHub or Analytics Engine failed | Same, for each upstream |
| `clock-skew` | Browser clock moved ±1 day | Dates and "x ago" text stay sane |

CLIs and games get the same idea in CI only (no production to break): CLIs
with an unreachable `TELEMETRY_URL`, a read-only home directory and a
corrupt config; games with a dropped frame budget and a missing asset.

**Safety rules**, enforced by the runner, not by convention:

- **No load testing, ever.** Chaos runs are a handful of single-user
  sessions. The runner refuses to start more than one browser per project
  at a time.
- **Hard request budget.** Each nightly run may send at most 500 requests to
  our Workers in total (0.5% of the 100,000 daily free requests); the runner
  counts them and stops at the cap. Static asset requests are free and
  unlimited, so browser runs against Pages cost nothing.
- **Never touch third parties.** Auth0, PostHog, GitHub and RapidAPI are only
  ever blocked from the browser, never called by the chaos runner. homer's
  RapidAPI key is never in scope (free-tiers FT-002 already enforces that).
- **Only after green.** Chaos runs only for a project whose `cd` evals passed
  that night, so it never piles onto a broken deploy.
- **Kill switch.** Repo variable `EVALS_CHAOS` (`on` or `off`) checked by the
  workflow before anything runs. `off` until Phase 4 ships and Jim says go.
- **Free-tier guard.** The run first calls `free-tiers.sh usage`; if any
  budget is at its warning level it skips with a notice. `free-tiers.sh
  check` gains FT-006: chaos workflows must check `EVALS_CHAOS`, must not
  use a matrix wider than one job per project, and must not reference the
  RapidAPI or Auth0 management secrets.

### Judge evals

Some requirements can't be checked by a script: "recipes read like a home
cook wrote them", "the review deck's summary is something Jim can act on".
A `judge` eval gives an agent a rubric and the evidence, and it grades.

- A new skill, `judge-requirements`, run weekly by a Claude scheduled task.
  For each `judge` eval it opens production (or reads the latest UAT report's
  screenshots), applies the rubric, and records `pass` or `fail` with a
  two-sentence reason and the screenshot it looked at, through `evals
  publish` in a small workflow it dispatches.
- Judge results are **advisory**: they can make a requirement amber, never
  red on their own.
- **Calibration.** Where Jim has rated the same requirement for the same
  deploy, the engine records whether the judge agreed. Pane shows the
  agreement rate per rubric. A rubric under 80% agreement over its last 10
  ratings gets a `TODO (agent-generated)` to rewrite it.

### Closing the loop

The deploy-feedback plan already has a scheduled agent task after the
nightly run that files `FIX` items for failed projects. This plan widens it
into one skill, `eval-triage`:

- Reads `evals verdicts` and new feedback each morning after the 4:00 AM
  observe run.
- For each requirement newly **not met**: adds a `FIX` to the project's
  `ISSUES.md` naming the requirement ID, the failing evals and the evidence
  links, unless an open `FIX` already names that requirement.
- For each eval in `error`, or disputed by Jim: adds a `TODO
  (agent-generated)` to fix the eval.
- For a `cd` or `chaos` failure right after a deploy: says in the PR body
  which deploy it was and how to roll back (`push-main.yml` dispatch with
  `project` and the last good `ref`).
- Opens one draft PR with all of it and a short summary. If nothing changed,
  it does nothing.

When a `FIX` that names a requirement is merged, the next run shows whether
it worked: the requirement turns green or it doesn't.

### Process evals (later)

The same engine can grade our own process, from GitHub data alone, as
requirements of a `monorepo` project (`MONO-xxx`):

- Change failure rate: production deploys followed by a `FIX` or rollback
  within 48 hours.
- Lead time: PR opened to deployed.
- Time to restore: requirement red to green.
- Review load: PRs waiting on Jim, and how long.
- Eval health: share of evals in `error`, judge agreement, requirement
  coverage.

These feed pane's system signals and the deep project review. They come last
because they only mean something once the rest is running.

## Phases

Each phase ends with something Jim can see: a job summary, a PR comment, a
pane page or a `FIX` that an agent then works.

### Phase 0: foundations

- The requirements and evals conventions in `standards/docs/` (a new
  `evals.md`), and project prefixes in `naming.md`.
- The `evals` project with `model` and `cli` crates, `evals check` in
  `//:audit` (warnings only at first).
- `evals run`/`publish` for `cargo` and `playwright` with the job summary
  and JSONL artifact (no D1 yet).
- Draft `requirements.md` and `evals.toml` for cookbook, as a `needs-human`
  PR, tagging its existing Playwright and Rust tests.

### Phase 1: after CI

- The `evals` task in each shared task file, run after `ci` in
  `mise-projects.yml` for PRs, pushes and nightly CI.
- The D1 `evals` database, its schema migrations in the `evals` project, and
  `evals publish` writing to it (needs D1 permission on the Cloudflare
  token).
- Requirements and evals drafted for pane and cahokia (`needs-human` PRs).
- The requirement list in the sticky PR comment, once previews exist.

### Phase 2: after CD

Needs the `smoke` tasks and production smoke runs from deploy feedback,
phase 3.

- `cd` evals after `deploy` and `smoke`; `http` eval kind.
- `trycmd` and `script` kinds for CLIs and controlled_chaos, against the
  released binaries.
- The `eval-triage` skill and its daily scheduled task, replacing the
  planned nightly FIX task.
- `evals check` makes "every `must` has a production eval" an error.

### Phase 3: observability

Needs the `telemetry` Worker, Web Analytics and PostHog from deploy feedback,
phases 4 and 5.

- Synthetic traffic tagging in every runner, the PostHog snippet and the
  `telemetry` Worker.
- The `metric` kind with the Web Analytics, Analytics Engine, PostHog and
  GitHub sources; `daily-observe-evals.yml` and the post-deploy run.

### Phase 4: chaos in production

- The browser fault catalogue (`offline`, `slow-3g`, `asset-500`,
  `auth-down`, `third-party-blocked`, `storage-denied`), the `fault` kind
  and `nightly-chaos.yml` with the request cap, kill switch and free-tier
  check.
- FT-006 in `free-tiers.sh check`.
- The signed chaos header for our Workers, starting with pane's Worker
  (pane phase 3) and `telemetry`.
- CLI and game fault cases in CI.

### Phase 5: human feedback in pane

Needs pane phases 3 to 5 (Worker, Health page, `pane-edit` writes).

- Pane's Requirements view and Requirements health signal, reading D1
  through the pane Worker and using `evals/model` for verdicts.
- Rate, Dispute and Propose actions through `pane-edit` PRs into
  `feedback.jsonl` and `requirements.md`.
- "Rate these" on the Today page.
- The nightly load of `feedback.jsonl` into D1.

### Phase 6: judge evals

- The `judge-requirements` skill and its weekly scheduled task.
- Judge calibration against Jim's ratings, shown in pane.

### Phase 7: process evals

- `MONO-xxx` requirements and the GitHub-derived metrics above.
- Feed them into pane's system signals and the deep project review deck.

## Things Jim does

These become `TODO (human)` lines in `HUMAN_ISSUES.md` in the PR of the phase
that needs them, not before:

- Phase 0 onward: approve each project's `requirements.md` PR (they carry
  `needs-human`). These define what "working" means, so they are his call.
- Phase 1: add D1 Edit to the `CLOUDFLARE_API_TOKEN` permissions.
- Phase 2: schedule the `eval-triage` Claude task (daily, after 4:00 AM
  Pacific).
- Phase 3: a read-only PostHog personal API key as a repo secret, and
  Analytics Engine read on the Cloudflare token (already asked for in
  `HUMAN_ISSUES.md`).
- Phase 4: create the `EVALS_CHAOS_KEY` secret (repo and Worker), and set
  `EVALS_CHAOS=on` when he is happy with the first manual runs.
- Phase 6: schedule the weekly `judge-requirements` Claude task.

## Cost

All on approved vendors and free plans. Limits from
[free-tiers.md](../free-tiers.md).

| Piece | Free limit | Our use |
| --- | --- | --- |
| GitHub Actions (eval steps, chaos, observe workflows) | Unlimited standard runner minutes on a public repo | A few minutes per project per night |
| D1 (`evals` database) | 5 GB, 5M rows read and 100,000 written a day | Under 5,000 writes a day; reads by pane cached |
| Workers requests from chaos | 100,000 a day (shared) | Capped at 500 a night by the runner |
| Static asset requests from chaos and `cd` evals | Unlimited | - |
| Analytics Engine queries | 10,000 a day | Under 200 a day |
| Web Analytics GraphQL | Free | A few queries a day |
| PostHog API reads | Free plan rate limits; reads don't use the event allowance | A few queries a day; synthetic sessions are never recorded |
| Cron Triggers | 5 per account (budget 3) | None: all schedules are GitHub Actions |
| Claude scheduled tasks | Existing plan | One daily (`eval-triage`), one weekly (`judge-requirements`) |

The engine adds no vendor and nothing that bills. If D1 or Analytics Engine
hits a limit, the run still writes the job summary and the JSONL artifact
and marks its results `error`, so a limit shows as amber, never as a pass.

## Decisions for Jim

Approving the PR that adds this plan approves these (recorded in decision
0003). Say if any should change.

1. Requirements live in `<project>/docs/requirements.md` with stable IDs,
   evals in `<project>/evals.toml`, and requirement changes need Jim's
   approval.
2. Results are stored in Cloudflare D1 (new to us, but on Cloudflare and
   free), with the job summary as the always-available fallback.
3. Production chaos is limited to faults injected in our own browser
   sessions and, with a signed header, in our own Workers: no load tests, a
   500-request nightly cap, a kill switch that starts `off`.
4. Failing evals after CD file a `FIX` and offer a rollback; they don't roll
   back automatically. Revisit once `cd` evals have run green for a month.
5. Agent judge evals are advisory: amber at most. Jim's rating wins.
6. Our own eval and chaos traffic is tagged and excluded from analytics and
   PostHog.
7. The TODOs go in the root backlog, after the deploy-feedback items they
   build on. Phase 0 and 1 don't depend on anything, so moving them to
   priority is a one-line change if Jim wants them sooner.

## Not doing (yet)

- Load or stress testing. It would burn the shared Workers allowance and
  could look like an attack to Cloudflare.
- Automatic rollback (Decision 4).
- Faults inside Cloudflare's or a vendor's infrastructure (taking a Pages
  project down, revoking a token). Those test the vendor, not our product.
- A/B tests or feature flags. PostHog's free flags could do it later.
- LLM evals of model output. No project calls a model today; when one does,
  its evals use the same `judge` and `metric` kinds.
