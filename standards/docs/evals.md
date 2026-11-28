# Requirements and evals

How a project writes down its product requirements and the evals that check
them. The why and the full design are in the
[eval engine plan](./plans/2026-11-eval-engine.md) and
[decision 0003](./decisions/0003-eval-engine.md). This page is the
convention: the two file formats every project uses and the rules
`evals check` enforces.

Each project that ships something has two files:

| File | Holds | Who decides |
| --- | --- | --- |
| `<project>/docs/requirements.md` | What the project promises its users, one requirement per heading | Jim. A PR that adds or changes a requirement is labelled `needs-human` and waits for his approval. |
| `<project>/evals.toml` | How each requirement is checked, at which stages | Anyone. Agents add and fix evals freely. |

Both live in the project's top-level directory (the one with its
`mise.toml`), so `cookbook/docs/requirements.md` covers `cookbook/web` and
`cookbook/cloudflare` too.

## Continuous improvement

Evals are an ongoing process, not a project with an end (Jim,
2026-11-27): "The evals should always be green and a system of feedback to
our todo process should be the way in which individual projects improve
their eval coverage."

- **Green is the normal state.** A requirement that is not met or at risk
  is a bug in the product or in its evals, and gets a `FIX` (product) or
  `TODO` (eval) issue file in its project's `docs/issues/`. A `must` that stays
  unknown (no production result) gets a `TODO` to add a production eval.
- **The loop files the work.** Eval triage reads `verdicts.json` and files
  those issues; agents work them like any other issue. Nothing is left for
  someone to notice.
- **Coverage grows through issues.** Every project that ships something has
  requirements and evals. Each project's eval coverage (how many of its
  requirements are green on production evidence) is measured and shown as
  a badge ([eval coverage](#eval-coverage)), and a gap becomes an issue,
  the same way code coverage works
  ([testing.md](testing.md#code-coverage)).
- **Work includes its evals.** A change that adds or alters what a project
  promises adds or updates the eval that checks it in the same PR.
- **Production evals check content.** An `http` eval of a page or data file
  checks what the response says (`body_contains`), not only that it
  answered: a single-page app answers every path.

The phases in the [plan](./plans/2026-11-eval-engine.md) are the order the
engine's *capabilities* arrive (metrics, chaos, ratings, in-app feedback,
judge); the improvement loop above runs all the time, from today.

## Requirements

### Format

```markdown
# Cookbook requirements

## COOK-001: A recipe is easy to cook from on a phone

- Priority: must
- Why: Jim cooks from his phone, one hand free.
- Accept: no sideways scrolling at 375 px wide; body text at least 16 px;
  the recipe is readable within 2.5 s on a 4G phone (LCP p75).

## COOK-002: Recipes can be found by ingredient

- Priority: should
- Why: Jim plans meals around what is already in the fridge.
- Accept: searching "leek" lists every recipe with leeks within 1 s.

## COOK-003: Recipes can be printed

- Priority: could
- Why: For cooking away from a phone.
- Accept: printing a recipe fits it on one page with no navigation.
- Status: retired (2026-12-01, nobody prints recipes)
```

Rules:

- **One `##` heading per requirement**, written as
  `## <PREFIX>-<nnn>: <title>`. Other `##` headings are not allowed in the
  file; put notes above the first requirement.
- **The ID** is the project's prefix (see [Project prefixes](#project-prefixes)),
  a hyphen, and a number of at least three digits: `COOK-001`, `COOK-042`,
  `COOK-1000`. Numbers go up in order of writing; gaps are fine.
- **IDs are never reused or renumbered**, even after a requirement is
  retired, so old results and comments still resolve.
- **The title** is one short sentence in the user's terms: what holds, not
  how it is built.
- **Fields** are list items directly under the heading, in this order:

  | Field | Required | Value |
  | --- | --- | --- |
  | `Priority` | yes | `must`, `should` or `could`. Only `must` requirements can turn a project red. |
  | `Why` | yes | Who needs it and why, in one or two sentences. |
  | `Accept` | yes | What "met" means, in words a person can check. Evals are written from it. A long value may wrap onto indented lines. |
  | `Status` | no | Absent means active. `retired (<yyyy-mm-dd>, <why>)` drops the requirement without deleting it. |

- **Retire, don't delete.** A dropped requirement keeps its heading and
  gains a `Status: retired (...)` line. Evals must not point at it.

### Who writes them

Requirements are Jim's product intent. An agent may draft them from the
project's README, its latest review deck's Goals slide, its plans and its
issue epics, but opens the PR with the `needs-human` label and never
merges it. A requirement PR changes only `docs/requirements.md` (and the
matching `evals.toml` entries), so Jim can approve it on its own.

## Evals

### Format

`<project>/evals.toml` starts with the project's prefix, then one
`[[eval]]` table per eval:

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
```

Fields every eval has:

| Field | Value |
| --- | --- |
| `id` | Unique within the project, kebab-case, stable. Results are keyed by it, so rename only with a reason. |
| `requirement` | An active requirement ID from the same project's `requirements.md`. |
| `stages` | One or more of `ci`, `cd`, `chaos`, `observe`, `judge`. |
| `kind` | One of the kinds below. |
| `description` | Optional. One line on what the eval checks, when the `id` doesn't say it. |

### Kinds

Each kind adds its own fields. The engine doesn't run `cargo`, `playwright`,
`trycmd` or `script` tests itself: it reads the reports the project's own
`ci`, `smoke` or `deploy` task already wrote, so tests run once.

| Kind | Checks | Extra fields | How tests are tagged |
| --- | --- | --- | --- |
| `cargo` | `cargo test` results | `filter` (optional, defaults to the requirement ID in snake case, `cook_001`) | Test function name starts with the snake-case ID: `fn cook_001_body_text_is_16px()` |
| `playwright` | Playwright JSON reporter results | `grep` (optional, defaults to `@req:<ID>`) | `@req:COOK-001` in the test title, alongside tags such as `@smoke` |
| `trycmd` | CLI transcript tests, from the `Testing <case> ... ok` lines trycmd prints inside the saved `cargo test` output | `glob` (optional, defaults to `tests/cmd/<ID>-*.toml`; `*` matches within one path segment) | Case file named `tests/cmd/COOK-001-<what>.toml` |
| `script` | A Bevy scripted run | `script`, `check` | A named check in the script file |
| `http` | One request | `url` (or `path`, joined to `DEPLOY_URL`), `method` (default `GET`), `status` (default 200), optional `header` (`"name: value"`: the value contains it, ignoring case), `body_contains`, `asset_contains` (see [Checking a live page](#checking-a-live-page)) and `max_ms` (the slowest acceptable response, body included) | Declared inline |
| `metric` | A query against observability data | `source` (`web-analytics`, `analytics-engine`, `posthog`, `github`), `metric`, `min` and/or `max`, `window` (`1h`, `7d`, ...) | Declared inline |
| `fault` | A Playwright test run with a fault injected | `fault` (a name from the plan's fault catalogue), `grep` | `@req:<ID>` in the test title |
| `judge` | An agent grades the live product | `rubric`: what passes and what fails, in plain words | Declared inline |
| `human-check` | Jim checks a setting made by hand | `setting`: an `id` in `standards/settings.toml` (see [Human checks](#human-checks)) | Declared inline, `stages = ["human"]` |
| `cost` | The project's cost analysis holds (see [Cost evals](#cost-evals)) | None | Declared inline, stages `ci` and/or `observe` only |

A `human` eval (Jim's rating in pane) exists implicitly for every
requirement and is never listed. Only `human-check` evals list the `human`
stage.

### Stages

| Stage | Runs | Against |
| --- | --- | --- |
| `ci` | Every PR, push to `main` and nightly CI | The build in the runner |
| `cd` | Right after each production deploy | The deployed site or released binary |
| `chaos` | Nightly, after that night's `cd` evals passed | Production, with a fault injected |
| `observe` | Daily, and one hour after each deploy | Observability data over the eval's `window` |
| `judge` | Weekly | Production and the latest UAT report |
| `human` | When Jim rates in pane, or answers a due human check | His own use, or the dashboard itself |

`cd`, `chaos` and `observe` are the **production stages**. Every `must`
requirement needs at least one eval in a production stage.

### Cost evals

A `cost` eval ties a requirement such as `COOK-010: Cookbook runs at $0 a
month` to the project's `docs/cost.toml` ([cost.md](cost.md)), so a cost
regression shows in the same verdicts as any other broken promise:

```toml
[[eval]]
id = "runs-free"
requirement = "COOK-010"
stages = ["ci", "observe"]
kind = "cost"
```

- **`ci`:** `cost check` writes the result itself (`cost check
  --evals-results <file>`), so the cost rules run once. In CI, `evals.sh
  run` calls it from the top-level project's own job. The eval fails when
  `cost check` finds anything `--strict` would fail on (COST-001 to
  COST-007) in the project's own files; the warning-only codes (COST-008 to
  COST-010) are listed in the note but pass, and account-wide findings
  belong to no one project.
- **`observe`:** the weekly actuals compared with the file's expected
  usage. Not collected yet (cost check phase 4).

## Checks

`evals check` (the `evals` project, `cargo run -p evals_cli -- check` in
`evals/`) checks every top-level project with an `evals.toml` or
`docs/requirements.md`. `mise run //:audit` runs it through
`standards/scripts/evals-check.sh` once any project has one of those files
(so it builds nothing before then), and so do the git hooks and PR CI.
Until Phase 4 of the plan the findings are warnings and never fail the
audit (`--strict` makes the command itself fail); then EVL-004 becomes an
error, and the rest follow once every shipping project has its files.

| Code | Finding |
| --- | --- |
| EVL-001 | An eval names a requirement that doesn't exist or is retired, or a `human-check` names a setting that isn't in `standards/settings.toml`. |
| EVL-002 | Two requirements or two evals in a project share an ID. |
| EVL-003 | A `cargo`, `playwright` or `trycmd` eval matches no test in the project. |
| EVL-004 | A `must` requirement has no eval in a production stage. |
| EVL-005 | A `playwright.config.ts` (or `.js`) anywhere in the repo doesn't add `jimhillr-evals` to its browsers' user agent. |
| EVL-006 | An `http` eval `GET`s a `path`, expects a 2xx and checks no content (no `body_contains` or `asset_contains`). |

For EVL-003 a test matches when a Rust `fn` name in the project starts with
the eval's `filter`, a `.ts` or `.js` file contains its `grep`, or a file
path ends with its `glob`. `target/`, `node_modules/` and hidden folders
are not searched.

Malformed files (a heading that isn't `<PREFIX>-<nnn>: <title>`, a missing
field, an unknown priority, stage or kind, an `evals.toml` whose `prefix`
doesn't match the project's prefix, a bad `standards/settings.toml` entry,
or an answer line naming an unknown setting) are reported too.

## In CI

Stage `ci` runs in every project job of `mise-projects.yml` (PR CI, pushes
to `main` and nightly CI), through the shared tasks and
`standards/scripts/evals.sh`:

1. The `ci` task saves what the tests print: `cargo test` output to
   `target/evals-reports/cargo-test.log` (`evals.sh report cargo-test ...`;
   it holds the trycmd case lines too),
   and in Dioxus web apps Playwright's JSON report to
   `target/evals-reports/playwright.json` (the `e2e` task adds
   `--reporter=list,json`). `target/` is ignored by git.
2. The `evals` task (`mise run evals`) runs `evals run` for the top-level
   project's `evals.toml` with those reports, appending to
   `$EVALS_RESULTS`. A project with no `evals.toml` collects nothing. A
   sub-project's job (`cookbook/web`), or a top-level project that has
   sub-projects, passes `--partial`, so it reports only the evals its own
   tests cover; each part's job reports its share.
3. The workflow then writes the job summary table (`evals.sh publish`) and
   uploads the results file as a 14-day `evals-results-<project>` artifact.
4. On pushes to `main` and nightly CI (never pull requests, whose results
   are for unmerged code), a last job, `store-evals` (`evals-store.yml`), commits every
   project's results to the `evals-results` branch (`evals.sh store`; see
   [Results and verdicts](#results-and-verdicts)).

The eval step runs even when `ci` failed, so failing evals show, and it
never fails the job itself: failing tests already fail `ci`, and a bug in
the engine must not block a pull request. Run the same steps locally with
`mise run ci && mise run evals && bash ../standards/scripts/evals.sh publish`
from a project directory.

## After deploy

Stage `cd` runs in the same job, right after `mise run deploy` publishes a
project that has a `DEPLOY_URL` (pushes to `main`, Nightly CD and manual
redeploys; never pull requests, and not when `deploy-check.sh` skipped the
deploy). The `cd evals` step first runs `evals.sh smoke`, then
`evals.sh run` with `EVALS_STAGE=cd`:

1. When the deploying project sets `SMOKE_DIR` in its `mise.toml` `[env]`
   (cookbook/cloudflare sets `"../web"`), `evals.sh smoke` runs that Playwright
   project's `@smoke` tests in desktop Chromium against `DEPLOY_URL`. Its
   `playwright.config.ts` reads `BASE_URL`: when set, it starts no dev
   server and records every test's video, screenshot and trace (its user
   agent says `jimhillr-evals`, as in every Playwright run). The JSON report goes to
   `target/evals-reports/cd/playwright.json` in the deploying project, and
   everything is kept as the 7-day `smoke-<project>` artifact. A failing
   smoke test only warns in this step; the evals below decide the job.
   Tag a test `@smoke` alongside its `@req:<ID>`, and give its `playwright`
   eval the `cd` stage.
2. Each `http` eval of the top-level project makes its request against the
   live site, `path` joined to `DEPLOY_URL`. Redirects are followed unless
   the eval expects a 3xx `status`. A miss is tried again, up to 3 times 5
   seconds apart, since a fresh deploy can take a moment to reach every
   edge. A site that never answers fails; an eval with a `path` and no
   `DEPLOY_URL` is an `error`. Requests send the user agent
   `jimhillr-evals/0.1 (synthetic check)` and the header
   `x-jimhillr-synthetic: 1`, so analytics and the `telemetry` Worker skip
   them. Playwright runs (CI, smoke and UAT) add `jimhillr-evals` to every
   browser's user agent through the `synthetic()` helper in each
   `playwright.config.ts` (EVL-005); a browser can't add the header to a
   beacon, so the user agent is what marks it.
3. Report kinds (`cargo`, `trycmd`, `playwright`) read only
   `target/evals-reports/cd/`, where the smoke run against production
   writes its report, never the `ci` task's reports.
4. A failed eval of a `must` requirement fails the job (`evals run
   --fail-on-must`), so a broken deploy shows red on main. The site is
   already live by then: roll back by running **Push Main CI/CD** by hand
   with the project and a good ref ([mise.md](mise.md)). An `error`, or a
   bug in the engine (exit status 2), only warns.
5. The results go in the job summary and a 14-day
   `evals-results-cd-<project>` artifact, and `store-evals` commits them to
   the `evals-results` branch with the `ci` ones (Nightly CD stores only its
   `cd` results, since Nightly CI stores that night's `ci` ones).

Run the same checks by hand from a deploying project's directory:
`mise run smoke`, then
`EVALS_STAGE=cd DEPLOY_URL=https://jimhillr-cookbook.pages.dev bash ../../standards/scripts/evals.sh run`.
Outside CI, `evals.sh smoke` skips with a note when it can't install Playwright's
browser (a cloud sandbox), and cloud sandboxes can't reach `*.pages.dev`.

### Checking a live page

cookbook, cahokia and pane are client-rendered Dioxus apps on Cloudflare
Pages. Their release has no `404.html`, so Pages serves the same
`index.html` shell, with status 200, for every path, `/recipe/1` and
`/no-such-page` alike. An `http` eval that only checks the status of a page
path proves nothing about that page, which is why EVL-006 warns about one.
Check content instead, in one of three ways:

- **A data file the page reads**, with `body_contains`: pane's
  `/data/snapshot.json`, bear_down's `/rooting.json`.
- **The bundle the shell ships**, with `asset_contains`: the eval fetches the
  page, then every same-origin `<script src>`, stylesheet, `preload` and
  `modulepreload` it names, and the absolute `.wasm` paths those scripts
  load (the hashed `web_bg-<hash>.wasm` from `dx bundle`). Each must answer
  200, and one must contain the text. Content compiled into the app (a
  recipe title, a sample person, the Auth0 tenant) is in the wasm as plain
  text, so this catches a deploy whose bundle is missing, stale or broken.
- **A server-rendered page**, with `body_contains` on the page itself.

Neither proves the page renders in a browser: that is a Playwright smoke run
against production (`cahokia/docs/issues/smoke-uat-tag-few-playwright-tests.md`).

## Human checks

Some settings are made by hand in a dashboard and live in no file an eval
can read: Claude connectors and scheduled tasks, repo secrets, Auth0
applications, Cloudflare token scopes, free-plan billing. They break
silently (an expired connector sign-in, a token missing a scope, a dropped
callback URL), so Jim checks each one on a schedule. The design is in the
plan's [Human checks](./plans/2026-11-eval-engine.md#human-checks-settings-made-by-hand)
section.

### The manifest

`standards/settings.toml` lists every hand-made setting once:

```toml
[[setting]]
id = "cloudflare-api-token-scopes"
vendor = "Cloudflare"
where = "dash.cloudflare.com > My Profile > API Tokens"
expect = "The token in CLOUDFLARE_API_TOKEN has Cloudflare Pages: Edit."
why = "A token missing a scope fails every deploy."
every = "90d"
```

| Field | Value |
| --- | --- |
| `id` | Unique, kebab-case, stable: answers are keyed by it. |
| `vendor` | Who hosts it: `GitHub`, `Claude`, `Cloudflare`, ... |
| `where` | The dashboard page, as a click path or a link. |
| `expect` | What it must be, in words Jim can check in under a minute. |
| `why` | What breaks without it. |
| `every` | How often to check it, in whole days: `30d`, `90d`. |

It never holds a secret's value, only that the secret exists and what it
is for; `evals check` rejects any other field. A human task
(`docs/issues/human/`) that creates a setting adds its entry here in the same PR.

### Answers

Jim's answers are his own judgment, so like pane ratings they live on
`main`: one JSON line per answer in `standards/settings-checks.jsonl`,
appended by `evals human-checks answer` and never edited by hand.

```json
{"setting":"claude-connectors","status":"pass","note":"ok","at":"2026-12-07T15:02:00Z","by":"jim"}
```

### The loop

1. **Due.** `evals human-checks due` (in `evals/`:
   `cargo run -q -p evals_cli -- human-checks due`) lists each setting that
   was never checked, whose last answer is older than its `every`, or whose
   last answer was "not ok":

   ```text
   Check claude-connectors: GitHub and Cloudflare are connected ... (Claude: claude.ai > Settings > Connectors). Reply ok or what is wrong. (Periodic, every 30d. Last checked 2026-11-02.)
   ```

2. **Asked.** The weekly `human-issues-digest` run (Mondays) runs it with
   `--write`, which writes a human task, `standards/docs/issues/human/settings-check-<id>.md`, for
   up to five due checks that have none yet (`--limit`), opens one PR with
   them, and lists every open check in the digest under "Checks due".
3. **Answered.** When Jim replies "ok", or says what is wrong, the agent
   that reads it runs `evals human-checks answer <id> --ok` or
   `evals human-checks answer <id> --not-ok "<his words>"`, which appends
   the answer and deletes the task file, and opens one PR. A "not ok" also
   files a `kind: fix` issue in the owning project's `docs/issues/`, or a new
   human task when only Jim can fix it, and the check comes up again until
   a newer "ok".

### Linking a requirement

An eval of kind `human-check` ties a requirement to a setting, so the
requirement's verdict can use the answer:

```toml
[[eval]]
id = "auth0-callbacks"
requirement = "CAHO-004"
stages = ["human"]
kind = "human-check"
setting = "auth0-cahokia-app"
```

A `human-check` eval runs in stage `human` only, and only it may list that
stage. A `setting` that isn't in the manifest is EVL-001. A setting no eval
names is fine: most are repo-wide and have no requirement yet.

The verdicts read the answers from `main`, not from the results:

- Jim's latest answer for the setting is "not ok": the requirement is
  **not met** (amber for a `should` or `could`, like any failure), until a
  newer "ok".
- The setting was never checked, or its latest "ok" is at least twice its
  `every` old: the requirement is **at risk**, never met, even before it
  has any production results.

## Results and verdicts

Results are files, never a database: JSONL in the job summary artifact and
on the long-lived `evals-results` branch, which shares no history with
`main`:

```text
results/<project>/<yyyy-mm>.jsonl       one line per result, appended
results/<project>/archive/<yyyy>.jsonl.gz  months older than 12
verdicts.json                           every active requirement's verdict
badge.svg                               the repo's eval coverage badge
badges/<project>.svg                    each project's eval coverage badge
```

`standards/scripts/evals.sh store <results.jsonl...>` (the
`evals-store.yml` job) checks the branch out in a worktree, appends the results
(`evals store`), moves months older than 12 into their year's archive,
rewrites `verdicts.json` and the eval coverage badges from the
requirements, evals and human-check answers on `main`, and pushes one commit, `evals: <stage> <projects> run
<id>`, retrying up to five times when another run pushed first. Its first
run creates the branch.

Each requirement gets one verdict (met, at risk, not met, unknown) computed
by the `evals/model` crate, with thresholds in `evals/verdicts.toml`. The
rules are in the plan's
[Results](./plans/2026-11-eval-engine.md#results) and
[Verdicts](./plans/2026-11-eval-engine.md#verdicts) sections, plus the
human-check rules above. `verdicts.json` lists each project's requirements
with their verdict and reasons, how many active `must` requirements it
has and how many have an eval in a production stage, and its [eval
coverage](#eval-coverage); the file's top-level `coverage` is the repo's.

To see the current verdicts, run
`bash standards/scripts/evals.sh verdicts` from the repo (it fetches the
branch), with `--project <p>` for one project or `--json` for the same
shape as `verdicts.json`:

```text
cookbook: 4 met, 1 at risk, 0 not met, 4 unknown (3 of 3 musts evaluated in production; eval coverage 44%, musts 100%)
  COOK-001 met      There is a recipe for every day of the year (must)
  COOK-004 at-risk  The cookbook is easy to use on a phone (should): mobile-layout is failing in ci
homer: no requirements yet (eval coverage 0%, warning)
eval coverage: 22%, musts 100% over 2 projects (1 with requirements); 50% of musts met per project on average, warning below 80% (warning)
```

### Eval coverage

Eval coverage says how much of what a project promises is green on
production evidence (Jim, 2026-11-27: "Maybe we introduce an eval coverage
metric / badge"). It is the measure the [continuous
improvement](#continuous-improvement) loop grows, the way code coverage is
for tests.

- **A project's coverage** is the share of its active requirements whose
  verdict is met, with its `must` requirements shown separately: `44%,
  musts 100%`. A met verdict already needs a fresh production-stage
  result, so a requirement checked only in `ci`, or with no eval, doesn't
  count. Every top-level project (a folder with a `mise.toml`) is listed,
  and one with no `docs/requirements.md` shows `0%, no requirements`.
- **The repo's coverage** is the mean of the projects' coverage, so a
  project with no requirements pulls it down instead of being left out,
  with the musts met over all musts beside it.
- **The warning** goes by the musts: a project with under `coverage_warn`
  percent (80, in `evals/verdicts.toml`) of its musts met, or of every
  requirement when it has no musts, is a warning, and so is the repo when
  the mean of those figures over every project is under it. It is a
  warning, never a failure, like code coverage under 80% (COV-001).
- **Badges.** `evals store` writes `badge.svg` (the repo) and
  `badges/<project>.svg` on the `evals-results` branch on every store run
  (each push to `main` and nightly). They are green from `coverage_warn`,
  yellow from three quarters of it and red below. The repo badge is in the
  "Repo health" table of the root README
  ([repo-health.md](./repo-health.md)), and each project's README has its
  own under its title:

  ```markdown
  [![Eval coverage](https://raw.githubusercontent.com/jimhillr/monorepo/evals-results/badges/<project>.svg)](https://github.com/jimhillr/monorepo/blob/evals-results/verdicts.json)
  ```

  A new project adds that line to its README when it is created.
- **A gap is an issue.** A project under the warning level gets `TODO`
  issues to write requirements or add production evals, and a requirement
  that isn't met gets a `FIX`, as in [continuous
  improvement](#continuous-improvement).

The `evals-results` branch is never merged into `main` and must never be
deleted, by `all-delete-branches.yml` or by hand.

## Triage

Green is the normal state, so a requirement that isn't green becomes an
issue file, and the normal agent loop works it. A free GitHub workflow
files them, not a Claude session, so a quiet day costs nothing (Jim,
2026-11-27, decision 1 of the
[executive review](../../docs/reviews/2026-11-27-evals/review.md)).

`evals-triage.yml` runs daily at 4:30 AM Pacific, after Nightly CI and
Nightly CD have stored their results, and runs
`bash standards/scripts/evals.sh triage`. That computes every verdict
again from the `evals-results` branch and `main` (`evals triage`) and, in a
worktree of `main`, writes one issue file in `<project>/docs/issues/` for each
requirement that needs work and that no open issue file names (any
`docs/issues/` folder in the project, or `standards/docs/issues/`, by its ID):

| Verdict | Issue | File |
| --- | --- | --- |
| Not met, or at risk because it failed in a production stage (or Jim said so) | `fix`, priority | `eval-<id>-failing.md`: the failing evals, their evidence links and the last commit they passed at, for a rollback |
| At risk for any other reason: an eval errors, is flaky, fails in `ci`, or has gone stale | `todo` (priority for a `must`, else backlog) | `eval-<id>-eval-at-risk.md`: fix the eval, not the product |
| Unknown, for a `must` (no production evidence) | `todo`, priority | `eval-<id>-unproven.md`: add or run a production-stage eval |

A `should` or `could` that is only unknown gets nothing, and neither does a
requirement whose only problem is a human check Jim hasn't done: that is
his, through the human-checks digest. It writes at most 10 files a run
(`--limit`); the next run files the rest.

When it wrote any, it commits them as `github-actions[bot]` and
force-pushes that one commit to the `evals-triage` branch (its only push,
SEC-007), and the next PR triage run opens a draft pull request from it, so
PR CI runs as usual and Jim reviews it like any agent PR. While that pull
request is open, triage files nothing new; once it merges, the issues are
open and the next run skips them. On a day with nothing to file it pushes
nothing, and deletes a leftover `evals-triage` branch. Run it by hand with
the workflow's **dry run** input, or locally with
`bash standards/scripts/evals.sh triage --dry-run`, to see what it would
file.

The `eval-triage` skill is for the judgment calls the
workflow can't make (a disputed eval, rollback advice); it is not a daily
task.

## Fire drill

The **evals fire drill** workflow (`evals-fire-drill.yml`, run by hand with a
project and an `http` eval, default cookbook `recipe-page-live`) proves the
loop still catches a failure. `evals drill --dir <evals-results checkout>`
points the eval at text the live page doesn't have, runs it against the
site, adds the failure (in memory only) to the stored results, and checks
the requirement turns red (amber for a `should`), that triage would file it
(a dry run of `evals triage`), and that after the real eval passes the
verdict is back where it started. It only reads: production gets the GET
requests a `cd` run makes, and nothing is written or pushed. Run it after
changing the verdict rules, `evals store` or triage. The first drill's retro
is [evals/docs/fire-drill-2026-11-27.md](../../evals/docs/fire-drill-2026-11-27.md).

## Project prefixes

Every project's requirement prefix is listed in
[naming.md](./naming.md#requirement-prefixes). Add a project's prefix there
in the same PR as its first `requirements.md`.
