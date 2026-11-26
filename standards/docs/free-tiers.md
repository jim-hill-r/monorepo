# Free tiers

Everything the repo runs on is used on a free plan, and automation must never
push us past one or onto a paid plan (Jim, 2026-11-26). This page lists the
free limits of each service, the budget we hold ourselves to, and the guards
that enforce it. Vendor choice itself is covered by the vendor rule in
[AGENTS.md](../../AGENTS.md).

Limits were checked on 2026-11-26 from each vendor's own pricing and limits
pages (linked below). They change; check the source before relying on a
number, and update this page in the same PR when one moves.

## Rules

1. **Never turn on a paid plan, add a payment method or raise a spending
   limit.** The one paid service is Claude Code, with a budget Jim set of
   $25 a month; every technology's budget is in the
   [tech radar](tech-radar.md), and every other budget is $0. No agent, workflow or script may upgrade a plan, enter a card,
   buy an add-on, or raise a GitHub budget or cache limit. Only Jim does
   that, and only after a decision record in [decisions/](decisions/README.md).
   A PR that needs one is labelled `needs-human`.
2. **Stop, don't pay.** When a limit is near, automation fails loudly and
   does nothing more. It never works around a limit by creating a second
   account, project or key.
3. **Budget well inside the limit.** Our budgets (below) leave room for
   mistakes and for Jim's own use of the same accounts.
4. **Create resources only through a guard.** Anything automation creates on
   demand (today: Cloudflare Pages projects) goes through
   `standards/scripts/free-tiers.sh`, which counts what exists first.
5. **Prefer services that hard-stop.** On a free plan with no payment method,
   every service below stops or throttles rather than billing. Keep it that
   way: a new service whose free plan bills overage automatically (for
   example RentCast) is not used.

## Limits and budgets

### GitHub (Actions, Dependabot, CodeQL)

Source: [How use of GitHub Actions is measured](https://docs.github.com/en/billing/concepts/product-billing/github-actions).

| Item | Free limit | Our budget | Past the limit |
| --- | --- | --- | --- |
| Minutes on standard GitHub-hosted runners | Unlimited on a public repo | - | - |
| Larger runners (custom labels, `-xlarge`, `-16core` and so on) | Billed even on public repos | Never used | Billed |
| Actions cache | 10 GB per repo | Warn at 8 GB | Oldest entries evicted; billed only if the limit is raised |
| Artifact and Packages storage | Free on a public repo (500 MB on the Free plan if the repo goes private) | Warn at 400 MB | Billed only with a payment method |
| Release assets (game downloads, `nightly-downloads.yml`) | Free; each file under 2 GiB, no total limit | One nightly prerelease per game, assets replaced in place | - |
| Minutes if the repo goes private | $12 a month of included compute (about 2,000 Linux minutes) | - | Blocked with no payment method |

The repo is public, so Actions costs nothing as long as jobs run on standard
runners (`macos-15` included; the game downloads use it). Making the repo private would change that; it needs a decision
record first.

### Cloudflare (Pages, Workers, observability)

Sources: [Pages limits](https://developers.cloudflare.com/pages/platform/limits),
[Workers limits](https://developers.cloudflare.com/workers/platform/limits/),
[Workers pricing](https://developers.cloudflare.com/workers/platform/pricing/),
[Observability pricing](https://developers.cloudflare.com/observability/pricing/),
[Analytics Engine pricing](https://developers.cloudflare.com/analytics/analytics-engine/pricing).

The Workers Free plan has no billing at all: past a limit, requests fail
(error 1027) or data stops being ingested until the daily reset at 00:00 UTC.
The risk is an outage, not a bill, unless someone moves the account to
Workers Paid, which rule 1 forbids.

| Item | Free limit | Our budget | Past the limit |
| --- | --- | --- | --- |
| Pages projects | 100 per account (hard, not routinely raised) | 25 | Creation fails |
| Pages builds (Cloudflare's own CI) | 500 a month, 1 at a time | 0: we deploy with `wrangler pages deploy` (direct upload), which uses no builds | - |
| Pages files per site / file size | 20,000 / 25 MiB | - | Deploy fails |
| Pages deployments kept | 30 days plus the 100 most recent | - | Older ones deleted |
| Workers per account | 100 | 25 | Deploy fails |
| Workers requests (includes Pages Functions) | 100,000 a day | Rate-limit public endpoints such as `telemetry` | Error 1027 until 00:00 UTC |
| Workers CPU time | 10 ms per request | - | Error 1102 |
| Cron Triggers | 5 per account | 3 | Deploy fails |
| Static asset requests | Unlimited | - | - |
| KV | 100,000 reads and 1,000 writes a day, 1 GB | - | Operation errors |
| D1 | 5M rows read and 100,000 written a day; 10 databases of up to 500 MB, 5 GB in total | - | Operation errors |
| R2 | 10 GB-month, 1M Class A and 10M Class B ops a month | - | R2 bills past this once enabled, so don't enable R2 without a decision record |
| Workers Logs (until 2026-12-01) | 200,000 events a day, 3 days | - | Stops ingesting |
| Observability (from 2026-12-01) | 0.5 GB a day, 7 days | - | Stops ingesting until 00:00 UTC |
| Analytics Engine | 100,000 data points written and 10,000 queries a day | - | Not billed today |
| Web Analytics | Free | - | - |
| Browser Run | 10 browser minutes a day | - | - |

Settings that need Workers Paid (`usage_model`, `[limits]` above the free
CPU time, `logpush = true`) are rejected by the repo check.

### PostHog (session replay, decision 0001)

Source: [PostHog pricing](https://posthog.com/pricing).

The free plan needs no card and has one project. Monthly allowances: 1M
product analytics events, 5,000 web and 2,500 mobile session recordings, 1M
feature flag requests, 100,000 exceptions, 1,500 survey responses. PostHog's
pages don't say whether the free plan drops data or stops at an allowance;
with no card on file it cannot bill. Adding a card turns on pay-as-you-go
above the allowance, so rule 1 applies. When PostHog is added to the sites,
record only on production hosts and sample recordings if traffic grows past a
few thousand sessions a month.

### Auth0 (sign-in for cahokia and cookbook)

Source: [Auth0 pricing](https://auth0.com/pricing).

The free plan needs no card: 25,000 monthly active users, 1 tenant, 1 custom
domain (adding it asks for a card, so don't), 1,000 machine-to-machine tokens
a month, 1 day of logs. Free tenants are not upgraded automatically past the
MAU limit. CI must not fetch real Auth0 tokens; tests use mocks so the M2M
allowance isn't spent by automation.

### RapidAPI (Zillow data for homer, free plan only)

homer calls the Zillow API on RapidAPI with a key the user supplies. It runs
by hand only: the repo check rejects any workflow that uses the RapidAPI key,
so no scheduled run can spend the quota. The free plan's monthly request
quota is shown on the API's RapidAPI pricing page and in the
`x-ratelimit-requests-limit` response header. homer saves the quota each
response reports, warns when 10% or less is left, and stops before a request
once it is used up (see homer's README, "Free quota").

### Squarespace

Jim's existing account, not used by automation. Nothing here creates or
changes anything in it.

### Claude Code (Anthropic)

Our AI coding platform and the one paid service: budget $25 a month (Jim,
2026-11-26). Jim's own subscription; no agent, workflow or script changes
its plan or adds usage-based billing.

## Guards

All guards live in [`standards/scripts/free-tiers.sh`](../scripts/free-tiers.sh)
and are tested in `monorepo/workflow_tests/tests/free_tier_tests.rs`.

| Guard | When it runs | What it does |
| --- | --- | --- |
| `free-tiers.sh check` | Every CI run, as part of `mise run //:audit` | FT-001 only standard runners; FT-002 no workflow uses the RapidAPI key; FT-003 Pages projects and Workers in the repo within budget; FT-004 Cron Triggers within budget; FT-005 no paid-only wrangler settings |
| `free-tiers.sh pages-project` | Before every Cloudflare Pages deploy (`mise run deploy` in a `cloudflare` project) | Creates the Pages project named in `wrangler.toml` only if it is missing and the account is under the Pages budget; otherwise stops the deploy |
| `free-tiers.sh usage` | Weekly, from [All Free Tier Usage](../../.github/workflows/all-free-tier-usage.yml), and (planned) first in the nightly run before it deploys | Reads live counts (Pages projects, Workers, Actions cache and artifacts), writes a table to the run summary, and fails when any reaches its warning level |

Budgets can be lowered with environment variables
(`FREE_TIERS_PAGES_PROJECTS_BUDGET`, `FREE_TIERS_WORKERS_BUDGET`,
`FREE_TIERS_CRON_TRIGGERS_BUDGET`), never raised past the free limit; the
script clamps them.

### When a guard fires

1. Don't raise the budget to make it pass. Find what used the room: an
   unused Pages project, an old Worker, a cache that grew.
2. Remove what is unused through a workflow or a PR, never by hand.
3. If the budget really is too small, propose a new one in a PR that
   updates this page and the script, and label it `needs-human`.

## Things only Jim can check

Agents can't see billing pages. Jim checks these; open checks are
files in the root [issues/human/](../../issues/human/):

- No payment method on the Cloudflare, PostHog, Auth0 or RapidAPI accounts,
  and GitHub's Actions budget is $0 with "stop usage" on.
- The Cloudflare account is on Workers Free, not Workers Paid.
- The `CLOUDFLARE_API_TOKEN` secret has no Billing permission. For the
  usage check it needs Pages read (it already has Pages edit) and Workers
  Scripts read.
- The RapidAPI Zillow plan is the free one, and its monthly quota.
