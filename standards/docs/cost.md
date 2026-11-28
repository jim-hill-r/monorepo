# Cost analysis (`docs/cost.toml`)

Every project keeps a structured cost analysis in
**`<project>/docs/cost.toml`**, in its top-level directory, next to
`docs/requirements.md` (so `cookbook/docs/cost.toml` covers `cookbook/web`
and `cookbook/cloudflare`). It says which outside services the project
uses, on which plan, how much of each free limit it expects to use, and any
money that is the project's own. `cost check` holds it to the budgets in the
[tech radar](tech-radar.md) and [free-tiers.md](free-tiers.md). The design
is in the [cost check plan](plans/2026-11-cost-check.md); how actual
usage, money and overrun risk are tracked across every vendor is in the
[cost tracking plan](plans/2026-11-cost-tracking.md).

A project that uses no outside service (a library, a CLI that runs locally)
still has the file, with `reviewed`, `summary` and nothing else, so "no
file" always means "not analysed yet", never "free". Prose goes in `notes`
fields, or in an optional `docs/cost.md` that the check ignores.

This file replaces the `finance.toml` pane's plan proposed: a project has one
money file, not two.

## Format

```toml
# cookbook/docs/cost.toml
reviewed = "2026-11-26"           # last time someone checked this file against the code
summary = "Static site on Cloudflare Pages with Auth0 sign-in. $0 a month."
notes = "Optional prose."

[[service]]                       # one per outside service the project uses
blip = "Cloudflare Pages"         # a vendor blip in standards/radar/radar.json
plan = "free"                     # "free", or "paid" only for a blip with a budget
why = "Hosts the site; deploys by direct upload, so no Pages builds."
[service.usage]                   # expected use; keys from standards/radar/limits.toml
pages_projects = 1
workers_requests_per_day = 2000   # Pages Functions count as Workers requests

[[service]]
blip = "Auth0"
plan = "free"
why = "Sign-in."
[service.usage]
auth0_mau = 5

# Money that is this project's own. Whole US cents, so nothing is lost to
# floating point.
[[cost]]
name = "cookbook.example.com domain"
vendor = "Squarespace"            # a vendor blip in radar.json
cents = 1200
every = "year"                    # "month", "year" or "once"
since = "2026-12"                 # first month it is paid, YYYY-MM
renews = "2027-12-01"             # next charge, YYYY-MM-DD; required for "year"
decision = "standards/docs/decisions/0006-cookbook-domain.md"  # required when cents > 0

[[revenue]]
month = "2026-11"
cents = 0
source = "None yet"
```

| Field | Rule |
| --- | --- |
| `reviewed` | `YYYY-MM-DD`. Update it whenever you check the file against the code. |
| `summary` | One or two sentences: what the project runs on and what it costs a month. |
| `[[service]].blip` | The exact name of a `vendor` blip in [radar.json](../radar/radar.json), in Adopt or Trial. |
| `[[service]].plan` | `free`. `paid` only for a blip whose radar budget is above $0 (today only Claude Code). |
| `[[service]].usage` | Expected use at the project's normal level, keyed by `key` in [limits.toml](../radar/limits.toml). Leave out what you can't estimate; a guess with a note beats nothing. |
| `[[cost]]` | `name`, `vendor` (a vendor blip), `cents`, `every`, `since`, and `decision` when `cents > 0`. |
| `[[cost]].renews` | `YYYY-MM-DD`, the next charge. Required when `every = "year"`, optional for `month`, not allowed for `once`. Move it on a year (or month) after each renewal; `cost renewals --within 60` lists the ones coming up. |
| `[[revenue]]` | `month`, `cents`, `source`. |

Unknown fields are errors, so a typo can't hide a cost.

## Which services to list

List a service when the project's own files use it: a `wrangler.toml`
(Cloudflare Pages or Workers), an Auth0 or PostHog SDK, a vendor API it
calls, a GitHub Release it publishes to, or a scheduled workflow of its own.
Plain open-source dependencies and the shared PR and nightly CI every
project gets cost nothing and are not listed. The detectors below warn when
a project's files use a service its `cost.toml` doesn't list (COST-007).

Shared free resources are counted where they are used: two projects that
each deploy a Pages site each list `pages_projects = 1`. A resource two
projects share (the one Auth0 tenant) is not listed as usage by either.

## Accounting rules

Jim, 2026-11-27: "All of our cost software should follow GAP accounting so
that everything is double counted on the ledger and also we should have
attribution in such a way that if you sum up all service costs/revenue it
equals total revenue/costs." So every piece of cost, money or ledger code
here follows these rules ([decision 0016](decisions/0016-holding-and-handling-money.md)):

1. **Double entry.** Every money movement is an entry whose postings sum
   to zero: each amount leaves one account and enters another (`income:`,
   `expense:`, `asset:`, `liability:`, `equity:`). An entry that doesn't
   balance is an error, never rounded away.
2. **Accrual and matching**, as GAAP asks: a cost is booked in the months
   it serves (a yearly cost spread over twelve months, a once cost in its
   month), revenue in the month it is earned, whatever day the cash moved.
   The bank reconciliation shows the cash side.
3. **Full attribution.** Every cent of revenue and cost belongs to exactly
   one project, and the projects sum, to the cent, to the totals. A shared
   cost is split by a stated rule (today: share of merged PRs, with
   repo-wide PRs and months with none charged to `standards`); leftover
   cents go by largest remainder, so there is no unattributed remainder.
4. **Whole cents** (integers), so nothing is lost to floating point.
5. **Consistency and disclosure.** A changed rule is a decision record,
   and every number links to its source.

The checks: pane's `attribution_gaps` (PANE-013) proves the per-project
money adds up to the month's totals; the double-entry balance check comes
with the financial ledger
([issue](../../cost/docs/issues/ledger-double-entry-balance-check.md)).

## The check

`cost check` (the [`cost`](../../cost/README.md) project) runs from
`mise run //:audit` through `standards/scripts/cost-check.sh`, so every
hook and CI run sees it. Until phase 3 of the plan every finding is a
**warning** and the audit still passes.

| Code | Fires when |
| --- | --- |
| COST-001 | A project has no `docs/cost.toml`. |
| COST-002 | The file doesn't parse, or has an unknown field, plan, `every` or usage key, or a yearly `[[cost]]` has no `renews` (or a one-off one has). |
| COST-003 | A `[[service]]` or `[[cost]]` names a blip that isn't a vendor in `radar.json`, or one in Assess or Hold. |
| COST-004 | `plan` isn't `free` for a blip whose budget is $0, or a `[[cost]]` with `cents > 0` has no `decision` pointing at an existing, accepted decision record. |
| COST-005 | Expected usage is over our budget for a limit (or the free limit, where we have no budget). Limits with `scope = "account"` are summed across every project. |
| COST-006 | This month's money for a vendor blip, summed over every project (yearly costs spread over twelve months), is over that blip's radar budget. |
| COST-007 | **Drift:** the project's files use a service its `[[service]]` list doesn't name, or more of a counted limit (Pages projects, Workers, Cron Triggers, D1 databases) than its `[service.usage]` expects. See "Detectors" below. |
| COST-008 | `reviewed` is more than 180 days old. A warning even after phase 3. |
| COST-009 | An `https://` (or `wss://`) host in the project's source or config that no detector or allow entry knows. A warning even after phase 3, until the allowlist settles. |
| COST-010 | A `[[cost]]`'s `renews` date has passed: the file wasn't updated after the renewal. Check what was charged and set the next date. A warning even after phase 3. |

The per-project rules (COST-001 to COST-004 and COST-007 to COST-010) run
only for the affected projects in the git hooks and PR CI: `hooks.sh` and
the PR's audit step pass the list in `COST_PROJECTS`. The account-wide rules
read every project's file every time, which takes milliseconds.

A requirement can depend on this check through an eval of kind `cost` in
the project's `evals.toml`: in stage `ci`, `cost check --evals-results`
writes its result in the eval engine's format, failing on COST-001 to
COST-007 in the project's own files. See
[Cost evals](evals.md#cost-evals).

### Detectors

[`standards/radar/detectors.toml`](../radar/detectors.toml) maps what is in
a project's files to the vendor blip it implies, and sometimes to a limit it
counts. The detectors read the project's whole tree (git's tracked files,
skipping `docs/`, `docs/issues/`, build output and lockfiles), not the diff, so a
hook, PR CI and `main` get the same answer, and a project can't drift
through a series of small PRs. Comment lines don't count.

| In the project's files | Implies |
| --- | --- |
| a `wrangler.toml` with `pages_build_output_dir` | Cloudflare Pages, `pages_projects` +1 |
| a `wrangler.toml` with `main` | Cloudflare Workers, `workers` +1 |
| `[triggers] crons` in a `wrangler.toml` | Cloudflare Workers, `cron_triggers` + the number of crons |
| `d1_databases`, `kv_namespaces`, `r2_buckets`, `analytics_engine_datasets` bindings | Cloudflare D1 (`d1_databases` + the count), Workers (KV, Analytics Engine), R2 |
| `posthog-js` or `posthog` in a manifest, or a `posthog.com` host | PostHog |
| an `@auth0/` package or `auth0` crate, an `auth0.com` host or an `AUTH0_` name | Auth0 |
| a `rapidapi.com` host, a `RAPIDAPI_` name or an `X-RapidAPI-` header | RapidAPI (Zillow data) |
| a workflow named `<project>.yml` or `<project>-*.yml` with a `schedule:` | GitHub Actions minutes |
| any other `https://` host | COST-009 |

A detector can also carry a `threat` field saying what a match means for
the project's threat model; the threat check reads the same matches for
THR-004 ([security.md](security.md#threat-models)). A detector with a
`threat` but no `blip` (a deploy URL, `wrangler secret put`, raw HTML, CORS
open to every origin) matters only to the threat check and costs nothing.

Links a person follows (an HTML or rsx `href`, a Markdown link) are not
services, so they never raise COST-009. When a host costs nothing (a docs
site, a registry, a free CDN), add it to `[[allow]]` with a `why`. When a
detector matches a file that doesn't use the service (a guard that searches
for a key, a test fixture), add an `[[exempt]]` for that file. Both need a
reason, and review checks it.

### The PR cost summary

PR CI's **Cost summary** job runs `cost summary --base origin/<base>`,
which compares the PR with its merge base and writes to the job summary:

- **Cost files changed:** any `docs/cost.toml`, `limits.toml`,
  `radar.json`, `detectors.toml` or `wrangler.toml`; a workflow line with
  `schedule:`, `cron:` or `runs-on:`; a `Cargo.toml` or `package.json`
  line a detector matches.
- **What moved:** per project, services added or dropped, usage and its own
  money this month before and after; account-wide usage against each budget
  with the headroom left; radar budgets and rings, and `limits.toml` budgets
  and free limits.

It labels the PR:

- `cost` when anything above changed. Nothing more is needed when the
  check passes; the job takes the label off again if a later push undoes it.
- `needs-human` when the PR adds a vendor blip to a project (in its
  `cost.toml`, or in code its `cost.toml` doesn't list), adds or raises a
  `[[cost]]`, moves a service to a `paid` plan, raises a budget in
  `radar.json` or `limits.toml`, or moves a vendor into Adopt or Trial.
  These are Jim's decisions (see "Cost approval" below), so PR triage won't
  merge the PR on green checks alone. The job never removes `needs-human`.

Run it yourself before pushing:

```bash
cargo run --quiet --manifest-path cost/Cargo.toml -p cost_cli -- summary --base origin/main
```

Run the check yourself:

```bash
bash standards/scripts/cost-check.sh                        # every project
bash standards/scripts/cost-check.sh --projects cookbook    # per-project rules for one
```

The free limits and budgets themselves live in
[`standards/radar/limits.toml`](../radar/limits.toml). The tables in
[free-tiers.md](free-tiers.md) are generated from it (`mise run fix` in
`standards`), and `standards` CI fails when they differ.

## Cost approval

Any expense, however small, is Jim's decision (the vendor and free-tier
rules in [AGENTS.md](../../AGENTS.md)). To add one:

1. Write a decision record in [decisions/](decisions/README.md) that says
   what it costs **a month and a year**, why the free tier (or an approved
   free vendor) isn't enough, and **how to cancel** it.
2. Add the `[[cost]]` entry to the project's `docs/cost.toml` with
   `decision` pointing at that record.
3. Label the PR `needs-human` and don't merge it until Jim accepts the
   decision record (its `Status:` line reads `Accepted`). COST-004 fails
   until then.

The same goes for a `paid` plan, a new vendor for a project, or raising a
budget in `radar.json` or `limits.toml`: only Jim sets budgets, so those
PRs are labelled `needs-human` too. Usage estimates and already-approved
free services merge on green checks like any other change.
