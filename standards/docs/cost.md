# Cost analysis (`docs/cost.toml`)

Every project keeps a structured cost analysis in
**`<project>/docs/cost.toml`**, in its top-level directory, next to
`docs/requirements.md` (so `cookbook/docs/cost.toml` covers `cookbook/web`
and `cookbook/cloudflare`). It says which outside services the project
uses, on which plan, how much of each free limit it expects to use, and any
money that is the project's own. `cost check` holds it to the budgets in the
[tech radar](tech-radar.md) and [free-tiers.md](free-tiers.md). The design
is in the [cost check plan](plans/2026-11-cost-check.md).

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
| `[[revenue]]` | `month`, `cents`, `source`. |

Unknown fields are errors, so a typo can't hide a cost.

## Which services to list

List a service when the project's own files use it: a `wrangler.toml`
(Cloudflare Pages or Workers), an Auth0 or PostHog SDK, a vendor API it
calls, a GitHub Release it publishes to, or a scheduled workflow of its own.
Plain open-source dependencies and the shared PR and nightly CI every
project gets cost nothing and are not listed. Phase 2 of the plan adds
detectors that warn when a project's files use a service its `cost.toml`
doesn't list (COST-007).

Shared free resources are counted where they are used: two projects that
each deploy a Pages site each list `pages_projects = 1`. A resource two
projects share (the one Auth0 tenant) is not listed as usage by either.

## The check

`cost check` (the [`cost`](../../cost/README.md) project) runs from
`mise run //:audit` through `standards/scripts/cost-check.sh`, so every
hook and CI run sees it. Until phase 3 of the plan every finding is a
**warning** and the audit still passes.

| Code | Fires when |
| --- | --- |
| COST-001 | A project has no `docs/cost.toml`. |
| COST-002 | The file doesn't parse, or has an unknown field, plan, `every` or usage key. |
| COST-003 | A `[[service]]` or `[[cost]]` names a blip that isn't a vendor in `radar.json`, or one in Assess or Hold. |
| COST-004 | `plan` isn't `free` for a blip whose budget is $0, or a `[[cost]]` with `cents > 0` has no `decision` pointing at an existing, accepted decision record. |
| COST-005 | Expected usage is over our budget for a limit (or the free limit, where we have no budget). Limits with `scope = "account"` are summed across every project. |
| COST-006 | This month's money for a vendor blip, summed over every project (yearly costs spread over twelve months), is over that blip's radar budget. |
| COST-008 | `reviewed` is more than 180 days old. A warning even after phase 3. |

Run it yourself:

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
