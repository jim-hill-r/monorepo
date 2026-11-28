# Plan: a cost check on every PR

Started 2026-11-26 from Jim's request:

> Can we create a ci/left hook action for cost analysis? Not sure how we can
> do this, but maybe it just looks in a designated docs folder in a project
> for a structured cost analysis and ensures that the analysis adheres to
> budget and standards. Therefore if a pr might impact cost we have a check
> for it.

Status: **approved** by Jim (PR #644, 2026-11-26), with the recommended
answer to every question (see "Decisions" at the end). The work items are
issue files in the root [issues/](../issues/) with `epic: Cost check`,
one group per phase.

## Goal

Every PR that could change what the repo costs is caught before it merges,
by the git hooks and again by PR CI, and the answer is written down where
people and tools can read it.

- Each project keeps a **structured cost analysis** in its `docs/` folder:
  which services it uses, on which plan, what it expects to use against each
  free limit, and what it costs a month (normally $0).
- A **check** proves the analysis is within the budgets and rules we already
  have (the [tech radar](../tech-radar.md) budgets, the
  [free-tier rules](../free-tiers.md), the vendor rule in
  [AGENTS.md](../../../AGENTS.md)).
- The check also proves the analysis is **true to the code**: if a project's
  files show it uses a service (a `d1_databases` binding, a PostHog snippet,
  a new cron) that its analysis doesn't list, the check fails.
- A PR that **raises** a cost, adds a vendor, or touches cost-sensitive
  files without updating the analysis is flagged, and a real cost increase is
  labelled `needs-human`, because only Jim sets budgets.
- The same files feed **pane's** Money page and the **eval engine**, so the
  numbers are written once.

## Where we start

| Piece | State on 2026-11-26 |
| --- | --- |
| Budgets | One per blip in [`standards/radar/radar.json`](../../radar/radar.json): $0 for everything except Claude Code ($25 a month). |
| Free limits | Prose tables in [free-tiers.md](../free-tiers.md); a few numbers hard-coded in `free-tiers.sh`. |
| Static guards | `free-tiers.sh check` (FT-001 to FT-005) in every `//:audit`: standard runners only, no RapidAPI key in workflows, Pages, Workers and Cron Trigger counts within budget, no paid-only wrangler settings. Repo-wide, not per project. |
| Live usage | `free-tiers.sh usage`, weekly in `all-free-tier-usage.yml`. |
| Per-project cost data | None. pane's plan proposes a `<project>/finance.toml` (revenue and project-only costs), not built yet. |
| Cost process TODOs | Root `issues/`: cost approval process, cost ledger (`standards/docs/costs.md`), monthly `review-costs` skill. |
| Hooks and PR CI | `hooks.sh` runs `//:audit` and `mise run ci` in affected projects only (PR #627); PR CI does the same once PR #642 merges (`AFFECTED_CHANGED_ONLY`). |
| Requirements | `<project>/docs/requirements.md` and `evals.toml`, checked by `evals check` as warnings (eval engine phase 0). |

So most of the rules exist; what is missing is a per-project record that the
check can hold each project to, and a way to tie a PR's changes to it.

## The cost analysis file

One file per project: **`<project>/docs/cost.toml`**, in the project's
top-level directory, next to `docs/requirements.md` (so
`cookbook/docs/cost.toml` covers `cookbook/web` and `cookbook/cloudflare`).
TOML because the check, pane and the eval engine all parse it; prose goes in
`notes` fields or in an optional `docs/cost.md`, which the check ignores.

```toml
# cookbook/docs/cost.toml
reviewed = "2026-11-26"           # last time someone checked this file against the code
summary = "Static site on Cloudflare Pages with Auth0 sign-in. $0 a month."

[[service]]
blip = "Cloudflare Pages"         # a vendor blip in radar.json
plan = "free"                     # "free" unless the blip has a budget
why = "Hosts the site; deploys by direct upload, so no Pages builds."
[service.usage]                   # expected use, keys from limits.toml
pages_projects = 1
workers_requests_per_day = 2000   # Pages Functions count as Workers requests

[[service]]
blip = "Auth0"
plan = "free"
why = "Sign-in."
[service.usage]
auth0_mau = 5

[[service]]
blip = "GitHub (Actions, Dependabot, CodeQL)"
plan = "free"
why = "CI, nightly CI and CD on standard runners."

# Money that is this project's own: replaces pane's proposed finance.toml.
# Whole US cents, so nothing is lost to floating point.
[[cost]]
name = "cookbook.example.com domain"
vendor = "Squarespace"            # a vendor blip in radar.json
cents = 1200
every = "year"                    # "month", "year" or "once"
since = "2026-12"
decision = "standards/docs/decisions/0006-cookbook-domain.md"  # required when cents > 0

[[revenue]]
month = "2026-11"
cents = 0
source = "None yet"
```

A project that uses no outside service (a library, a CLI that runs
locally) still has the file, with `summary` and no `[[service]]`, so "no
file" always means "not analysed yet", never "free".

### Free limits as data

The limits and our budgets in [free-tiers.md](../free-tiers.md) move into
**`standards/radar/limits.toml`**, one entry per limit, keyed by the radar
blip it belongs to:

```toml
[[limit]]
key = "pages_projects"
blip = "Cloudflare Pages"
free = 100            # the vendor's free limit
budget = 25           # ours, well inside it
scope = "account"     # summed across every project
unit = "projects"
source = "https://developers.cloudflare.com/pages/platform/limits"
checked = "2026-11-26"
```

`free-tiers.sh` reads its budgets from this file instead of hard-coding
them, and the tables in `free-tiers.md` are generated from it the way
`radar.sh render` generates the radar tables, so a limit lives in one place.

## The check

A new command, `cost check`, written in Rust (the standards say script in
Rust, and pane and the eval engine will link the same model crate). It reads
only files in the repo, needs no credentials and no network, and takes well
under a second once built.

### Rules

| Code | Fails when |
| --- | --- |
| COST-001 | A project has no `docs/cost.toml`. |
| COST-002 | The file doesn't parse, or has an unknown field, plan, `every` or usage key. |
| COST-003 | A `[[service]]` or `[[cost]]` names a blip that isn't a vendor in `radar.json`, or one in Assess or Hold (the vendor rule: Jim signs off first). |
| COST-004 | `plan` isn't `"free"` for a blip whose budget is $0, or a `[[cost]]` has `cents > 0` with no `decision` pointing at an existing, accepted decision record. |
| COST-005 | A project's expected usage is over our budget for a limit, or, for `scope = "account"` limits, the sum across all projects is (25 Pages projects, 25 Workers, 3 Cron Triggers, 100,000 Workers requests a day, and so on). |
| COST-006 | The monthly money for a blip, summed over every project (yearly costs spread over twelve months), is over that blip's radar budget. |
| COST-007 | **Drift:** the project's files show it uses a service its `cost.toml` doesn't list. See detectors below. |
| COST-008 | `reviewed` is more than 180 days old (a warning, never a failure). |

COST-005 and COST-006 overlap FT-003 and FT-004 today. Once `cost check` is
failing on errors, FT-003 and FT-004 become thin wrappers that call it, so
there is one count, not two.

### Detectors: is the analysis true to the code?

COST-007 is what makes the check catch "a PR that might impact cost" without
anyone remembering to update a doc. A small table,
**`standards/radar/detectors.toml`**, maps file patterns to the blip (and
sometimes the limit) they imply:

| Pattern in the project's files | Implies |
| --- | --- |
| a `wrangler.toml` with `pages_build_output_dir` | Cloudflare Pages, `pages_projects` +1 |
| a `wrangler.toml` with `main =` | Cloudflare Workers, `workers` +1 |
| `[triggers] crons` in a `wrangler.toml` | `cron_triggers` + the number of crons |
| `d1_databases`, `kv_namespaces`, `r2_buckets`, `analytics_engine_datasets` bindings | Cloudflare D1, KV, R2, Analytics Engine |
| `posthog-js` or `posthog` in a manifest, or `i.posthog.com` in source | PostHog |
| `auth0` crates or packages, or an `AUTH0_` variable | Auth0 |
| `rapidapi.com` in source, or a `RAPIDAPI_` variable | RapidAPI (Zillow data) |
| a workflow under `.github/workflows/` whose name starts with the project's name and has a `schedule:` | GitHub Actions scheduled minutes |
| an `https://` host in source or config that no detector or allowlist entry knows | a warning: "unknown outside service `api.example.com`, add it to cost.toml or the allowlist" |

The detectors run over the project's **whole tree**, not the diff, so the
result is the same in a hook, in PR CI and on `main`, and a project can't
drift through a series of small PRs. Plain open-source dependencies cost
nothing and are ignored; only manifest entries that name a vendor's SDK
match. The unknown-host warning starts as a warning only, because it will be
noisy until the allowlist (docs sites, crates.io, CDNs that cost nothing) is
built up.

### Flagging PRs that might change cost

On top of pass or fail, PR CI compares the PR with its base and writes a
short **cost summary** to the job summary:

- **Cost files changed:** any `docs/cost.toml`, `limits.toml`,
  `radar.json` budget, `wrangler.toml`, workflow `schedule:` or `runs-on:`,
  or a manifest line a detector matches.
- **What moved:** blips added or removed, usage and money before and after,
  per project and account-wide, and headroom left against each budget.

Labels, set by the same workflow job (no new vendor, no token beyond the
`GITHUB_TOKEN` the job already has):

- `cost` when anything in the list above changed, so PR triage and Jim can
  see it at a glance. Nothing more is needed when the check passes.
- `needs-human` when the PR **raises** money (a `[[cost]]` with
  `cents > 0`, a budget in `radar.json` or `limits.toml` goes up, a `plan`
  that isn't free) or adds a vendor blip to a project. These are Jim's
  decisions under the vendor and free-tier rules, so PR triage won't merge
  them on green checks alone.

### Where it runs: fast and affected-only

The same split as PR #627 and #642:

- **Per-project rules** (COST-001 to COST-004, COST-007, COST-008) run only
  for affected projects. `hooks.sh` already computes the list, so it calls
  `cost check --projects <list>` once, right after `//:audit`. PR CI does the
  same with the list `affected.sh` gives it under `AFFECTED_CHANGED_ONLY`.
- **Account-wide rules** (COST-005, COST-006) need every project's
  `cost.toml`, but they only parse a few dozen small TOML files, so they run
  every time. That takes milliseconds.
- **Build cost:** the binary is built once and cached in the project's
  `target/` (as `evals-check.sh` does today), so a hook pays for the build
  only on the first run in a clone or after the cost crates change. If
  `cargo` is missing, the hook prints a warning and PR CI checks it, the
  existing sandbox fallback in [git-hooks.md](../git-hooks.md).
- **Not in the hook:** anything that needs the network or credentials (live
  usage, billing). That stays in the weekly `free-tiers.sh usage` run.

### Where the code lives

A new top-level project, **`cost`**, a Cargo workspace like `evals`:

- `cost/model`: the `cost.toml`, `limits.toml` and `detectors.toml` types,
  parsing, and pure functions (sum usage, spread yearly money, compare with
  budgets). No I/O beyond reading files it is given. pane and the eval
  engine depend on it by path.
- `cost/cli`: `cost check`, `cost summary --base <ref>` (the PR summary and
  labels decision as JSON) and `cost render` (the free-tiers.md tables).
- `standards/scripts/cost-check.sh`: the thin wrapper the audit and hooks
  call, the same shape as `evals-check.sh`.

A separate project rather than more crates in `evals` keeps a change to the
cost rules from running the evals CI, and the other way round.

## Reuse

### pane (Money page, Phase 7 of pane's plan)

pane's plan proposed `<project>/finance.toml`. This plan folds that file
into `docs/cost.toml` (its `[[cost]]` and `[[revenue]]` tables are the same
shape), so a project has one money file, not two. `pane_snapshot` reads
every `docs/cost.toml` through `cost/model` and adds to the snapshot:

- per project: services and plans, money this month, expected usage;
- account-wide: usage against each budget from `limits.toml`, and the latest
  weekly actuals from `free-tiers.sh usage` side by side with what the
  projects expected, so "Workers requests: expected 4,000 a day, saw
  11,000" shows up as a yellow light;
- shared costs (Claude Code) split by merged PRs as pane's plan already
  describes.

The pane TODOs that mention `finance.toml` get reworded to `docs/cost.toml`
in phase 1, and its "Add a cost" form appends to that file through a
`pane-edit` PR, which the check then verifies like any other.

### Eval engine

Cost becomes evidence like any other:

- A new eval `kind = "cost"` in `evals.toml`, at stages `ci` (the static
  check) and `observe` (the weekly actuals). Its verdict is met when
  `cost check` passes for the project and the weekly actuals are within the
  expected usage plus a margin.
- A project can then write a requirement such as
  `COOK-010: Cookbook runs at $0 a month` and tie it to that eval, so a cost
  regression shows up in the same verdicts, pane Health light and `FIX` line
  as any other broken promise.
- `cost check` writes its results in the eval engine's results format, so
  the engine doesn't need to call it twice.

### Existing cost TODOs in the root issues/

- **Cost approval process:** COST-004 is that check, using decision records
  with a `decision` link instead of parsing `Cost:` lines.
- **Cost ledger:** the ledger becomes a generated view of every
  `docs/cost.toml` plus the radar (`cost render --ledger`), not a hand-kept
  `costs.md`.
- **Monthly cost review:** unchanged, but it compares actuals with the
  projects' expected usage instead of a hand-kept ledger.

Phase 1 rewrites those three TODOs to match.

## Phases

### Phase 1: format, limits and the check as warnings

- `standards/docs/cost.md`: the `docs/cost.toml` convention, linked from
  `standards/docs/README.md`.
- `standards/radar/limits.toml` from free-tiers.md; `free-tiers.sh` reads
  its budgets from it; `cost render` generates the free-tiers.md tables and
  `standards` CI checks they match.
- The `cost` project with `model` and `cli`, and `cost check` with
  COST-001 to COST-006 and COST-008, run from `//:audit` as **warnings
  only** (the eval engine's phase 0 pattern), with unit tests for each rule.
- `docs/cost.toml` for the projects that deploy or call a vendor first
  (bear_down, cahokia, cookbook, homer, pane), then the rest.
- Reword the pane and root `issues/` TODOs as described under Reuse.

### Phase 2: detectors and the PR summary

Done 2026-11-26. COST-009 was added for unknown hosts, so COST-007 can
become an error in phase 3 while the unknown-host warning stays a warning.

- `standards/radar/detectors.toml` and COST-007, warnings only.
- `cost summary --base` and the PR CI step that writes the summary and sets
  the `cost` and `needs-human` labels.
- `hooks.sh` calls `cost check --projects <affected>`; `standards/tests`
  gains cases for it like the existing hook tests.

### Phase 3: make it fail

Once every project has a `docs/cost.toml` and the check has run quietly for
a week, COST-001 to COST-007 become errors in both the hook and PR CI.
FT-003 and FT-004 call `cost check` instead of counting on their own.

### Phase 4: reuse

- pane's Money page reads `docs/cost.toml` (pane Phase 7).
- The eval engine's `cost` kind, and `cost check` writing eval results.
- The weekly usage run compares actuals with expected usage and writes
  `standards/cost/usage/<yyyy-mm>.json` for pane and the monthly review.

## Cost of this plan

Nothing new: Rust on standard GitHub-hosted runners, files in the repo, the
existing weekly usage run. The hook gets one more step that takes well under
a second after the first build.

## Decisions

Jim, 2026-11-26: "Yes to all, use default guesses."

1. **File and place:** `<project>/docs/cost.toml`, machine-checked, with an
   optional `docs/cost.md` for prose.
2. **One money file:** pane's planned `finance.toml` is folded into
   `docs/cost.toml`.
3. **When to block:** warnings first; failures in phase 3, once every
   project has a file.
4. **Who decides:** any cost above $0, a new vendor for a project, or a
   budget change gets `needs-human` and waits for Jim; usage estimates and
   already-approved free services merge on green checks.

## Not doing (yet)

- Reading real bills. Agents can't see billing pages, and the free plans
  have no payment method; Jim's checks in
  [issues/human/](../issue-management.md#human-issues) cover that.
- Pricing paid plans or forecasting "what would it cost if we outgrew the
  free tier". Rule 2 in free-tiers.md says stop, don't pay; a decision
  record does that maths when it is needed.
- Costing Claude Code per PR. It is a flat $25 budget; pane splits it by
  merged PRs.
