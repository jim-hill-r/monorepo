# Plan: an eval engine for product requirements

Started 2026-11-26 from Jim's request:

> I'd like to build a robust eval engine into our processes. After ci, after
> cd, we have observability we can eval against, human feedback as a part of
> pane, and automated in production chaos testing to evaluate product
> requirements against as implemented.

The decision behind this plan is
[../decisions/0003-eval-engine.md](../decisions/0003-eval-engine.md). The work
items are issue files in the root [issues/](../../../issues/) with
`epic: Eval engine`, and in [pane/issues/](../../../pane/issues/) for the
pane pages, one group per phase below.

> **Update, 2026-11-27.** Jim: "The eval process is a continuous
> improvement process and should not be considered a one off project." The
> phases below are the order the engine's capabilities arrive; keeping
> every requirement green and growing coverage through issue files runs
> continuously. See "Continuous improvement" in
> [evals.md](../evals.md#continuous-improvement) and the
> [executive review](../../../docs/reviews/2026-11-27-evals/review.md).

> **Update, 2026-11-28.** Jim agreed with the whole review. The work left
> is now ordered by the [evals roadmap](../../../evals/docs/roadmap.md),
> and its issues are in [evals/issues/](../../../evals/issues/) under the
> epic "Evals roadmap". The phase 6 capture tooling moves to the user
> feedback component as one tool for every role.

## Goal

For every project, at any time, answer one question with evidence: **does
what we shipped do what we said it should?**

- Each project writes down its **product requirements**, each with an ID.
- Each requirement has one or more **evals**: checks that say pass or fail
  (or give a number against a threshold) for that requirement.
- Evals run at every point Jim named: **after CI**, **after CD**, against
  **observability** data, as **human feedback** (ratings in pane and
  comments pinned inside the apps and games themselves), and as **chaos
  tests in production**. An agent **judge** covers what can't be automated.
- Every run writes **results** to one place, so each requirement gets a
  verdict (met, at risk, not met, unknown) with a link to the evidence, and a
  history.
- A failing requirement becomes a `FIX` issue file in the project's `issues/`, so the
  existing agent loop works it. Nothing new to watch.

Tests already tell us the code does what the code says. Evals tell us the
product does what the requirement says, in production, for real users and
under failure. Most evals reuse tests we already have or plan (Playwright
`@smoke`, `trycmd`); the engine adds the link to a requirement, the
production stages, and the verdict.

## Where we start

| Piece | State on 2026-11-26 |
| --- | --- |
| Product requirements | None written down. Intent lives in READMEs, issue epics, plans and the review decks' Goals slide. |
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
requirements.md            evals.toml                 results (files)       verdicts
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
| `human-check` | Jim checks a hand-configured setting in a dashboard | `setting` names an entry in `standards/settings.toml`; see Human checks |

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
| `human-check` | When a check is due (its `every`), through the weekly human issues digest | The dashboard or account itself | Nothing; a "not ok" answer files a `FIX` |

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

No database (Jim, 2026-11-26: avoid databases for as long as possible and
keep dev work as files in the monorepo). Records go three places:

1. **The job summary**, as a table per project: requirement, eval, status,
   evidence. This works from Phase 0 with nothing else set up.
2. **A results file**, `evals-results.jsonl`, uploaded as a 14-day Actions
   artifact for debugging.
3. **Files committed to the monorepo**, on a long-lived data branch,
   `evals-results`, so the history is in git without a nightly PR or a
   commit on `main` for every run:

   ```
   results/<project>/<yyyy-mm>.jsonl   one line per result, appended
   verdicts.json                       current verdict per requirement
   ```

   Each run ends with a `store-evals` job (`evals.sh store`, which runs
   `evals store`), which checks out that branch, appends its records,
   recomputes `verdicts.json`, and pushes one commit
   (`evals: <stage> <projects> run <id>`), retrying on a race with another run.
   The workflow gets `contents: write` for that job only, and the push is
   to `evals-results` only; `main` is never written. At our scale (about 20
   projects, a few hundred evals, a handful of runs a day) that is well under
   1 MB a day before git's compression. Once a year a step moves months
   older than 12 into `results/<project>/archive/<yyyy>.jsonl.gz`.

   Pane, the triage skill and people all read these files: pane's snapshot
   fetches the branch at build time, and anyone can open the files on
   GitHub. The branch is never merged into `main` and must not be offered
   to `all-delete-branches.yml`.

Human feedback lives on `main` instead: it is Jim's own judgment, written by
pane's `pane-edit` PRs (see Human feedback), and the verdicts read it from
there.

**If a real query engine is ever needed** (for example for process evals over
a year of history), the next step is a SQLite file built from these JSONL
files and committed beside them, not a hosted database. A hosted database
comes only after the repo-wide database evaluation and its decision record.

### Verdicts

Each requirement gets one verdict, computed by the same Rust function in the
engine and in pane:

| Verdict | Rule |
| --- | --- |
| **Not met** (red) | The latest result of any eval in `cd`, `chaos` or `observe` failed, or Jim's latest rating since the last deploy is "not met". For a `should` or `could` requirement this shows as amber. |
| **At risk** (amber) | An eval failed 2 or more of its last 7 runs (flaky), a `metric` is within 10% of its threshold, the judge says not met, a `ci` eval is failing on main, or the newest production result is older than 3 days. |
| **Met** (green) | Every eval's latest result passes, and at least one production-stage result is under 2 days old. |
| **Unknown** (grey) | No evals, or no production-stage results yet, unless a `ci` eval is failing (then at risk: a known failure never shows grey). |

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
- `evals publish` — write the job summary from the JSONL results.
- `evals store` — add results to a checkout of the `evals-results` branch
  and rewrite its `verdicts.json`; `standards/scripts/evals.sh store` does
  the checkout, commit and push (built in phase 1).
- `evals verdicts [--project <p>]` — print the current verdicts from the
  `evals-results` files
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
  `jimhillr-evals` user agent and an `x-jimhillr-synthetic: 1` header. The
  PostHog snippet doesn't load for that user agent (so no recording is
  spent), the `telemetry` Worker drops events with either one (so metric
  queries never see them), and Web Analytics ignores headless browsers
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

The verdicts read these from `main`. A "not met" rating files a `FIX` with
Jim's note; a dispute files a `TODO (agent-generated)` to fix or remove the
eval. Pane's Today page lists requirements that have never been rated by
Jim and changed since his last rating, a few at a time, so feedback is a
two-minute habit rather than a chore.

### In-app feedback: pin a comment to the thing itself

Jim asked on 2026-11-26 for feedback inside the apps, games and websites,
like Vercel's toolbar: open the deployed product, click the spot that is
wrong, type a comment, and have it land where the work happens. Pane's
Rate button covers a requirement as a whole; this covers "this button,
on this screen, in this build".

**What Jim sees.** A small floating feedback button, shown only to him
(see Who can use it). Clicking it turns on comment mode: the page dims,
the element under the pointer is outlined, and a click drops a numbered
pin with a comment box. He picks a kind (**bug**, **idea**, **not met**,
**looks good**), can tie it to a requirement (the box suggests the
requirements whose evals cover the current page), and sends. Existing open
pins for the page show where they were left, with their replies, so he
doesn't comment on something twice. A pin can be replied to or resolved
from the toolbar or from pane.

**What each comment captures**, so an agent can reproduce it without
asking:

| Field | Web apps and sites | Desktop and mobile (webview) | Bevy games |
| --- | --- | --- | --- |
| Where | URL and route, a CSS selector path plus the element's text and role, the click's offset inside the element, viewport size, scroll position | Same, plus the app's route | Scene or state name, screen position, world position and the entity under the cursor (name or marker component) |
| Which build | `commit` and `deploy` from a `<meta name="jimhillr-build">` tag every build writes | App version and commit baked in at build time | Version and commit baked in, seed, frame number |
| Context | Screenshot of the viewport with the pin drawn on it, last 20 console errors, browser and OS | Screenshot, last 20 errors from the log, OS | Screenshot (Bevy's screenshot API), last 20 log errors, the RNG seed and recent inputs when the scripted mode from deploy feedback phase 3c exists |
| Who and when | `by: jim`, UTC time | Same | Same |

Pins re-anchor on the next build by selector, then text, then position, the
way Vercel's comments do; a pin whose element is gone shows as "moved" at
its old position rather than disappearing.

**One toolbar, three hosts.** Everything stays in Rust:

- **`feedback/toolbar`**: a small Rust crate compiled to wasm with
  `web-sys` (no framework), loaded by one `<script type="module">` tag.
  Because it doesn't depend on Dioxus or Sycamore, the same file works in
  every website (the Dioxus apps, the Perseus sites on Sycamore 0.8, review
  decks) and in **Dioxus desktop and mobile apps**, which render in a
  webview: their shells inject the same script into the webview.
- **`feedback/bevy`**: a Bevy plugin for controlled_chaos (and starcraft if
  it comes off hold). A hotkey (F8) or an on-screen button pauses the game
  and enters comment mode; a click picks the entity under the cursor
  through Bevy's picking. In the wasm build it uses the same send path as
  the toolbar; native builds use the same HTTP call from Rust.
- **`feedback/model`**: the shared `Comment`, `Anchor` and `Reply` types and
  the re-anchoring rules, used by the toolbar, the Bevy plugin, pane and the
  `evals` engine.

**Who can use it.** The toolbar is in every build but stays hidden and
inert for everyone else:

- It turns on with `?feedback` on the URL (or the hotkey in apps and
  games), remembered per device.
- Sending needs a feedback token. The toolbar opens a small pane window,
  `/feedback/sign-in`, which sits behind pane's Cloudflare Access (Jim
  only), and pane hands back a signed token (HMAC, 12-hour expiry) by
  `postMessage`. The toolbar keeps it in `sessionStorage` and sends it with
  each comment. A popup and a token, rather than cookies, because browsers
  block third-party cookies between the apps' domains and pane's.
- Desktop and mobile apps open the same sign-in page in the system browser
  and receive the token through their deep link.
- Public visitors never see the button, and the script loads no tracking.
  Real-user feedback, if wanted later, goes through PostHog's free surveys
  (1,500 responses a month), not this toolbar.

**Where comments go: files in the repo, no database.** The toolbar posts to
pane's Worker, `POST /api/feedback`, which checks the token and writes
through pane's existing write path, the `pane-edit` PR:

```
<project>/evals/feedback.jsonl              one JSON line per comment, reply or resolve
<project>/evals/feedback/<comment-id>.webp  the screenshot, downscaled to at most 1280 px and 200 KB
```

```json
{"type":"comment","id":"fb-20261202-1804-k3","at":"2026-12-02T18:04:00Z","by":"jim","project":"cookbook","kind":"bug","requirement":"COOK-001","text":"Header overflows on my phone","build":{"commit":"abc1234","deploy":"123456"},"anchor":{"url":"/recipes/42","selector":"header > nav","text":"Shopping list","offset":[0.92,0.4],"viewport":[390,844]},"screenshot":"feedback/fb-20261202-1804-k3.webp","errors":[]}
{"type":"reply","id":"fb-20261202-1804-k3","at":"2026-12-03T09:10:00Z","by":"agent","text":"Fixed in #640","pr":640}
{"type":"resolve","id":"fb-20261202-1804-k3","at":"2026-12-03T09:10:00Z","by":"agent","pr":640}
```

It is the same file as pane's Rate and Dispute actions, with a `type` on
every line. Comments sent in a burst go into the one open `pane-edit` PR, so
a review session is one PR. The toolbar reads open pins for the current page
from pane (`GET /api/feedback?project=&url=`), which serves them from its
snapshot of `main` plus the open `pane-edit` PR.

**How it feeds evals and work.**

- **not met** on a requirement counts as Jim's rating of that requirement
  for that deploy, so it can turn the requirement red (see Verdicts).
  **looks good** counts as a "met" rating.
- `eval-triage` turns each new **bug** or **not met** into a `FIX` in the
  project's `issues/`, and each **idea** into a `TODO`, quoting the
  comment and linking its screenshot and anchor. The PR that fixes it
  appends a `reply` and a `resolve` line, so the pin closes itself in the
  app and in pane.
- When a pin's requirement has no eval that would have caught it, triage
  also adds a `TODO (agent-generated)` to add one, so each piece of human
  feedback makes the automated evals stronger.
- The judge's calibration (see Judge evals) uses **not met** and **looks
  good** like ratings.

**Pane.** A Feedback page lists every comment, filterable by project, kind,
status and requirement, each showing its screenshot with the pin drawn on
it, the build, and an "Open in app" link that loads the URL with the toolbar
on and the pin highlighted. Each Project page shows its open pins.

**Cost.** No new vendor. A few pane Worker requests per comment, well inside
100,000 a day. Screenshots are capped at 200 KB, so even 50 comments a week
add under 10 MB a month to the repo; the yearly archive step moves
screenshots of resolved comments older than 12 months out of the tree (they
stay in git history).

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
   exist). A request carrying `x-jimhillr-chaos: <fault>` and a valid HMAC
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

### Human checks: settings made by hand

Some of what the products rely on is set up by hand in a dashboard
("clickops") and lives in no file an eval can read: Claude MCP connectors
and scheduled tasks, repo and Worker secrets, Auth0 application settings,
Cloudflare API token scopes, Access policies, Web Analytics, PostHog
project settings, and free-plan billing settings. Jim asked on 2026-11-26
for the eval loop to cover them too (they break silently: an expired
connector sign-in, a token missing a scope, a callback URL dropped).

- **A manifest**, `standards/settings.toml`, lists each hand-made setting
  once: `id`, `vendor`, `where` (the dashboard page or a link to it),
  `expect` (what it must be, in words Jim can check in under a minute),
  `why` (what breaks without it) and `every` (how often to check, such as
  `30d` or `90d`). It never holds a secret's value, only that the secret
  exists and what it is for. Adding a setting by hand without adding it
  here is the thing to avoid; human issues that create a
  setting add its manifest entry in the same PR.
- **Evals of kind `human-check`** in a project's `evals.toml` (or the
  `jimhillr` project's, for repo-wide settings such as Claude connectors)
  point a requirement at a setting: `kind = "human-check"`,
  `setting = "<id>"`, `stages = ["human"]`. `evals check` reports a
  `setting` that isn't in the manifest (EVL-001 widened) and a manifest
  entry no eval uses (a warning).
- **Due checks become human issues.** `evals verdicts` lists every
  `human-check` whose latest result is older than its `every`. The weekly
  `human-issues-digest` run (Mondays) adds each due one to
  `issues/human/` as a periodic task file, `Check <setting>: <expect>
  (<where>). Reply ok or what is wrong.` (periodic, every 30d, covers
  COOK-003), in one PR with the rest of its changes, and brings it
  to Jim with the rest of the digest.
- **Jim's answer is the result.** When Jim answers in the project ("ok", or
  what is wrong), the agent that reads it records an answer with
  `by = "jim"`, `status` `pass` or `fail` and his words, and deletes the
  task file. A `fail` also files a `FIX` (or a `TODO (human)` if only Jim can fix
  it). As built (2026-11-26), answers are Jim's judgment like pane ratings,
  so they live on `main` in `standards/settings-checks.jsonl`, keyed by
  setting and appended by `evals human-checks answer` (which also deletes
  `issues/human/settings-check-<id>.md`), rather than on the `evals-results` branch, which agents
  can't push to from a thread. Due checks come from
  `evals human-checks due --write`; `evals verdicts` will read the same
  file. A
  manifest entry that no eval names is not a warning: most settings are
  repo-wide and have no requirement until the `JIMH-xxx` ones exist.
- **Verdicts.** A failed human check makes its requirement not met, like a
  production eval. A check past twice its `every` with no answer makes it
  at risk, never met: an unchecked setting is unknown, not fine.


The deploy-feedback plan already has a scheduled agent task after the
nightly run that files `FIX` items for failed projects. This plan widens it
into one skill, `eval-triage`:

- Reads `evals verdicts` and new feedback each morning after the 4:00 AM
  observe run.
- For each requirement newly **not met**: adds a `FIX` to the project's
  `issues/` naming the requirement ID, the failing evals and the evidence
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
requirements of the `jimhillr` project (`JIMH-xxx`):

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
  and JSONL artifact (nothing committed yet).
- Draft `requirements.md` and `evals.toml` for cookbook, as a `needs-human`
  PR, tagging its existing Playwright and Rust tests.

### Phase 1: after CI

- The `evals` task in each shared task file, run after `ci` in
  `mise-projects.yml` for PRs, pushes and nightly CI.
- The `evals-results` data branch, `evals publish` committing results and
  `verdicts.json` to it, and the yearly archive step.
- Requirements and evals drafted for pane and cahokia (`needs-human` PRs).
- The requirement list in the sticky PR comment, once previews exist.

### Phase 2: after CD

Needs the `smoke` tasks and production smoke runs from deploy feedback,
phase 3.

- Done (2026-11-27): `cd` evals after `deploy`, and the `http` eval kind
  (with `max_ms`); a failed `must` fails the deploy job. They will read
  the `smoke` reports from `target/evals-reports/cd/` once projects have
  `smoke` tasks. See "After deploy" in [evals.md](../evals.md).
- `trycmd` and `script` kinds for CLIs and controlled_chaos, against the
  released binaries. Done (2026-11-27): the `trycmd` kind, collected from
  the saved `cargo test` output.
- Done (2026-11-27): eval triage, replacing the planned nightly FIX task.
  The free `evals-triage.yml` workflow files the issues daily (see
  "Triage" in [evals.md](../evals.md)); the `eval-triage` skill is for
  the judgment calls it can't make (rollback advice, a disputed eval),
  run on demand, not as a daily Claude task (Jim, 2026-11-27).
- `evals check` makes "every `must` has a production eval" an error.

### Phase 3: observability

Needs the `telemetry` Worker, Web Analytics and PostHog from deploy feedback,
phases 4 and 5.

- Synthetic traffic tagging in every runner, the PostHog snippet and the
  `telemetry` Worker. Done for the runners (Playwright configs, EVL-005,
  and the `evals` HTTP client) and the Worker; the PostHog snippet skips
  the `jimhillr-evals` user agent when session replay adds it.
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

- Pane's Requirements view and Requirements health signal, reading the
  `evals-results` files into its snapshot at build time and using
  `evals/model` for verdicts. Pane is redeployed after each eval run so the
  page is never more than one run behind.
- Rate, Dispute and Propose actions through `pane-edit` PRs into
  `feedback.jsonl` and `requirements.md`.
- "Rate these" on the Today page.
- Ratings from `feedback.jsonl` counted in the verdicts.

### Phase 6: in-app feedback

Needs Phase 5 (pane's Worker, Access and `pane-edit` writes).

- `feedback/model` (built as phase 1 of the
  [user feedback plan](2026-11-user-feedback.md), which also adds the public
  user widget, in-app help and a browser extension for this toolbar; the
  toolbar, webview and Bevy steps below moved to its Phase 8, one
  feedback tool with roles, Jim 2026-11-28), and
  pane's `/feedback/sign-in` token page and
  `/api/feedback` routes writing `feedback.jsonl` lines and screenshots
  through `pane-edit` PRs.
- `feedback/toolbar` (wasm), and the `jimhillr-build` meta tag, in cookbook
  first, then every web app and Perseus site.
- The toolbar injected into the cahokia desktop and mobile webviews, with
  sign-in through the deep link.
- `feedback/bevy` in controlled_chaos (native and wasm).
- `eval-triage` handling comments, and pane's Feedback page.

### Phase 7: judge evals

- The `judge-requirements` skill and its weekly scheduled task.
- Judge calibration against Jim's ratings, shown in pane.

### Phase 8: process evals

- `JIMH-xxx` requirements and the GitHub-derived metrics above.
- Feed them into pane's system signals and the deep project review deck.

## Things Jim does

These become human issue files in `issues/human/` in the PR of the phase
that needs them, not before:

- Phase 0 onward: approve each project's `requirements.md` PR (they carry
  `needs-human`). These define what "working" means, so they are his call.
- Phase 2: schedule the `eval-triage` Claude task (daily, after 4:00 AM
  Pacific).
- Phase 3: a read-only PostHog personal API key as a repo secret, and
  Analytics Engine read on the Cloudflare token (already asked for in
  `issues/human/`).
- Phase 4: create the `EVALS_CHAOS_KEY` secret (repo and Worker), and set
  `EVALS_CHAOS=on` when he is happy with the first manual runs.
- Phase 6: nothing new; the toolbar uses pane's existing Access policy.
- Phase 7: schedule the weekly `judge-requirements` Claude task.
- Human checks: answer each due "Check <setting>" line in the weekly
  digest with "ok" or what is wrong.

## Cost

All on approved vendors and free plans. Limits from
[free-tiers.md](../free-tiers.md).

| Piece | Free limit | Our use |
| --- | --- | --- |
| GitHub Actions (eval steps, chaos, observe workflows) | Unlimited standard runner minutes on a public repo | A few minutes per project per night |
| Results files on the `evals-results` branch | Free on a public repo (keep the repo well under GitHub's 1 GB guidance) | Under 1 MB a day uncompressed; archived yearly |
| Workers requests from chaos | 100,000 a day (shared) | Capped at 500 a night by the runner |
| Static asset requests from chaos and `cd` evals | Unlimited | - |
| Analytics Engine queries | 10,000 a day | Under 200 a day |
| Web Analytics GraphQL | Free | A few queries a day |
| PostHog API reads | Free plan rate limits; reads don't use the event allowance | A few queries a day; synthetic sessions are never recorded |
| Cron Triggers | 5 per account (budget 3) | None: all schedules are GitHub Actions |
| Claude scheduled tasks | Existing plan | One daily (`eval-triage`), one weekly (`judge-requirements`) |

The engine adds no vendor, no database and nothing that bills. If Analytics
Engine or PostHog hits a limit, the run still commits its results with those
evals marked `error`, so a limit shows as amber, never as a pass.

## Decisions for Jim

Approving the PR that adds this plan approves these (recorded in decision
0003). Say if any should change.

1. Requirements live in `<project>/docs/requirements.md` with stable IDs,
   evals in `<project>/evals.toml`, and requirement changes need Jim's
   approval.
2. Results are JSONL files committed to the monorepo on an `evals-results`
   data branch, not in a database (Jim, 2026-11-26). If one is ever needed,
   a committed SQLite file comes first.
3. Production chaos is limited to faults injected in our own browser
   sessions and, with a signed header, in our own Workers: no load tests, a
   500-request nightly cap, a kill switch that starts `off`.
4. Failing evals after CD file a `FIX` and offer a rollback; they don't roll
   back automatically. Revisit once `cd` evals have run green for a month.
5. Agent judge evals are advisory: amber at most. Jim's rating wins.
6. Our own eval and chaos traffic is tagged and excluded from analytics and
   PostHog.
7. In-app feedback is one Rust toolbar (wasm) for every website and
   webview app plus a Bevy plugin for games, usable only by Jim through a
   token from pane, storing comments and screenshots as files through
   `pane-edit` PRs.
8. The TODOs go in the root backlog, after the deploy-feedback items they
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
