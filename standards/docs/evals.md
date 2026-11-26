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
`ISSUES.md` epics, but opens the PR with the `needs-human` label and never
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
| `trycmd` | CLI transcript tests | `glob` (optional, defaults to `tests/cmd/<ID>-*.toml`) | Case file named `tests/cmd/COOK-001-<what>.toml` |
| `script` | A Bevy scripted run | `script`, `check` | A named check in the script file |
| `http` | One request | `url` (or `path`, joined to `DEPLOY_URL`), `method` (default `GET`), `status` (default 200), optional `header` (`"name: value"`) and `body_contains` | Declared inline |
| `metric` | A query against observability data | `source` (`web-analytics`, `analytics-engine`, `posthog`, `github`), `metric`, `min` and/or `max`, `window` (`1h`, `7d`, ...) | Declared inline |
| `fault` | A Playwright test run with a fault injected | `fault` (a name from the plan's fault catalogue), `grep` | `@req:<ID>` in the test title |
| `judge` | An agent grades the live product | `rubric`: what passes and what fails, in plain words | Declared inline |

A `human` eval (Jim's rating in pane) exists implicitly for every
requirement and is never listed.

### Stages

| Stage | Runs | Against |
| --- | --- | --- |
| `ci` | Every PR, push to `main` and nightly CI | The build in the runner |
| `cd` | Right after each production deploy | The deployed site or released binary |
| `chaos` | Nightly, after that night's `cd` evals passed | Production, with a fault injected |
| `observe` | Daily, and one hour after each deploy | Observability data over the eval's `window` |
| `judge` | Weekly | Production and the latest UAT report |

`cd`, `chaos` and `observe` are the **production stages**. Every `must`
requirement needs at least one eval in a production stage.

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
| EVL-001 | An eval names a requirement that doesn't exist or is retired. |
| EVL-002 | Two requirements or two evals in a project share an ID. |
| EVL-003 | A `cargo`, `playwright` or `trycmd` eval matches no test in the project. |
| EVL-004 | A `must` requirement has no eval in a production stage. |

For EVL-003 a test matches when a Rust `fn` name in the project starts with
the eval's `filter`, a `.ts` or `.js` file contains its `grep`, or a file
path ends with its `glob`. `target/`, `node_modules/` and hidden folders
are not searched.

Malformed files (a heading that isn't `<PREFIX>-<nnn>: <title>`, a missing
field, an unknown priority, stage or kind, or an `evals.toml` whose `prefix`
doesn't match the project's prefix) are reported too.

## In CI

Stage `ci` runs in every project job of `mise-projects.yml` (PR CI, pushes
to `main` and nightly CI), through the shared tasks and
`standards/scripts/evals.sh`:

1. The `ci` task saves what the tests print: `cargo test` output to
   `target/evals-reports/cargo-test.log` (`evals.sh report cargo-test ...`),
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

The eval step runs even when `ci` failed, so failing evals show, and it
never fails the job itself: failing tests already fail `ci`, and a bug in
the engine must not block a pull request. Run the same steps locally with
`mise run ci && mise run evals && bash ../standards/scripts/evals.sh publish`
from a project directory.

## Results and verdicts

Results are files, never a database: JSONL in the job summary artifact and
on the long-lived `evals-results` branch (`results/<project>/<yyyy-mm>.jsonl`
and `verdicts.json`). Each requirement gets one verdict (met, at risk, not
met, unknown) computed by the `evals/model` crate, with thresholds in
`evals/verdicts.toml`. The rules are in the plan's
[Results](./plans/2026-11-eval-engine.md#results) and
[Verdicts](./plans/2026-11-eval-engine.md#verdicts) sections.

The `evals-results` branch is never merged into `main` and must never be
deleted, by `all-delete-branches.yml` or by hand.

## Project prefixes

Every project's requirement prefix is listed in
[naming.md](./naming.md#requirement-prefixes). Add a project's prefix there
in the same PR as its first `requirements.md`.
