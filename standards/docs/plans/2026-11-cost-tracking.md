# Plan: tracking cost and overrun risk across everything

Jim, 2026-11-27: "In general we need an overall process for tracking costs
and potential costs and potential for overruns."

This plan ties the pieces we already have into one process and fills the
gaps. It doesn't replace them: the [cost check](2026-11-cost-check.md)
(`docs/cost.toml`, the PR cost summary), [free-tiers.md](../free-tiers.md)
(limits, budgets, guards), the weekly usage run, [repo health](../repo-health.md)
and PR triage, the weekly human issues digest and its settings checks
(`standards/settings.toml`), and the billing checks that the free-tier
payment check work is adding ([Billing checks](../free-tiers.md#billing-checks):
`standards/scripts/billing-check.sh`, run by the `billing` job in
`all-free-tier-usage.yml`). Jim, 2026-11-27: "the billing check should
check every vendor in the tech radar", so it covers every vendor blip in
`radar.json`, by API where the vendor has one (Cloudflare, GitHub, PostHog
today) and by Jim's settings check otherwise. This plan doesn't design
those billing checks; it says where their result goes.

## Goal

At any time, anyone (Jim, an agent, pane) can answer three questions from
the repo and its workflow runs, without opening a vendor dashboard:

1. **Actual cost:** what are we paying this month? Today the answer should
   be "Claude Code, within $25, and nothing else".
2. **Potential cost:** how close is each free allowance to running out, and
   which planned work would use more of it?
3. **Overrun risk:** what could start billing us, and would we notice
   within a day?

And every signal has an owner and a threshold, so a cost problem turns a
badge red and reaches Jim before it reaches a bill.

## Three kinds of cost

| Kind | What it is | Measured by | Today |
| --- | --- | --- | --- |
| Actual money | A charge that a vendor bills | `[[cost]]` in each `docs/cost.toml` (expected), billing checks and Jim's checks (actual) | Claude Code only; no `[[cost]]` entries anywhere |
| Free-tier headroom | Usage of a free allowance, which stops or throttles when used up | `[service.usage]` in `docs/cost.toml` (expected), `free-tiers.sh usage` (live) | All well inside budget (table below) |
| Overrun switches | A setting that would turn a free allowance into a bill: a payment method, a paid plan, an overage toggle, the repo going private | Guards that stop automation from flipping them, billing checks and Jim's checks that detect them | All believed off; only partly checked automatically |

Free-tier headroom running out is an **outage** risk, not a money risk, as
long as every overrun switch stays off: past a limit, the free plans fail
or throttle (free-tiers.md rule 5). So the money risk is concentrated in the
switches, and that is where detection matters most.

## Inventory

Everything the repo uses that is metered or could cost money. "Expected" is
the sum of every project's `docs/cost.toml` today. "Live" was read on
2026-11-27 with read-only API calls where an agent can see it. The weekly
usage run has not run yet (its first scheduled run is Monday 2026-11-30).

### Money

| Vendor | Plan | Budget | Expected this month | How the actual is measured | Status |
| --- | --- | --- | --- | --- | --- |
| Anthropic, Claude Code | Jim's subscription | $25 a month | $25 | Jim's 30-day settings check `claude-code-budget`. Agents can see whether a session is on overage usage (`rate_limit_info.isUsingOverage` from the session API), but not dollars. | Partly automatic (phase 2) |
| Squarespace | Jim's account, several paid domains (Jim, 2026-11-27) | $0 in the radar (to be set) | not recorded yet | Nothing yet: phase 4 lists each domain as a yearly `[[cost]]` and watches renewals | **Phase 4** |
| Every other vendor in the tech radar | Free | $0 | $0 | The billing check covers every radar vendor: by API where one exists, by Jim's check otherwise | See overrun switches |

### Free-tier headroom

| Resource | Free limit | Our budget | Expected | Live 2026-11-27 | Measured automatically by | Gap |
| --- | --- | --- | --- | --- | --- | --- |
| Cloudflare Pages projects | 100 | 25 | 7 | 10 (3 are the old names waiting for deletion, `infra/docs/issues/human/delete-old-pages-projects-after-jimhillr-rename.md`) | `free-tiers.sh usage` weekly; `pages-project` guard on every deploy | none |
| Cloudflare Pages builds | 500 a month | 0 | 0 | not read | nothing; we deploy by direct upload | low risk, FT checks the config |
| Cloudflare Workers | 100 | 25 | 2 | 1 (`jimhillr-tofu-state`; telemetry not deployed yet) | `free-tiers.sh usage`; `worker` guard on deploy | none |
| Workers requests (incl. Pages Functions) | 100,000 a day | none | not estimated | not read | nothing | **phase 1: GraphQL analytics** |
| Cron Triggers | 5 | 3 | 0 | not read | FT-004 counts them in the repo | live count in phase 1 |
| Cloudflare D1 databases | 10 | none | 1 | 1 | `cost check` (expected only) | live count and size in phase 1 |
| D1 storage, rows read and written | 5 GB; 5M and 100k a day | none | not estimated | not read | nothing | **phase 1** |
| KV, Analytics Engine, Workers Logs, Browser Run | daily limits | none | 0 | 0 KV namespaces | nothing | phase 1, once a project uses them |
| Cloudflare R2 | 10 GB | 0 (on hold) | 0 | not enabled (assumed) | FT checks no `r2_buckets` binding | R2 enabled is an overrun switch |
| GitHub Actions minutes | unlimited (public repo) | none | n/a | public repo | nothing | repo visibility is an overrun switch |
| GitHub Actions cache | 10 GB | 8 GB | n/a | not read yet | `free-tiers.sh usage` weekly | none |
| GitHub artifact storage | 500 MB if private | 400 MB | n/a | not read yet | `free-tiers.sh usage` weekly | none |
| GitHub release assets | 2 GB a file | one prerelease per game | n/a | n/a | the release workflow replaces assets | none |
| Auth0 MAU | 25,000 | none | 10 | not read | nothing | phase 3, needs a token |
| Auth0 M2M tokens | 1,000 a month | 0 (Jim to set, about 60) | 0 | not read | nothing | count our own token requests (phase 3) |
| PostHog events and recordings | 1M and 5k a month | none | 0 (not added yet) | n/a | nothing | `standards/docs/issues/when-posthog-added-extend-free-tiers.md` |
| RapidAPI (Zillow) requests | the free quota | stop at quota | n/a | n/a | homer saves the quota header and stops at zero | runs by hand only (FT-002); no change |
| Claude Code usage | the plan's allowance | $25 a month | n/a | not on overage (this session) | session API, overage flag | **phase 2** |

### Overrun switches

Each switch would turn a free allowance into money. "Prevent" is what stops
automation from flipping it; "Detect" is how we'd learn someone did.

| Switch | Prevent | Detect | Detect cadence |
| --- | --- | --- | --- |
| Payment method on Cloudflare | Tokens have no Billing write (free-tiers.md rule 1) | Billing check (`payment-methods`) | weekly |
| Cloudflare Workers Paid or any paid subscription | FT-005 rejects paid-only settings | Billing check (`subscriptions`) | weekly |
| R2 enabled | radar Hold, budget 0, FT checks | Billing check (`subscriptions`), and phase 1 reads `r2/buckets` | weekly |
| Payment method or subscription on PostHog | rule 1 | Billing check (`billing`, PostHog billing API) | weekly |
| GitHub Actions budget above $0, or a card | rule 1 | Billing check reads the month's net usage; the card and the budget setting have no API, so Jim's check | weekly, and 30 days |
| Repo made private (Actions minutes start counting) | needs a decision record | **phase 1:** the usage run reads `private` from the repo API | daily |
| Larger runners | FT-001 rejects them | FT-001 on every PR | every PR |
| Auth0 card or custom domain | budget 0 for custom domains | Jim's check | 30 days |
| RapidAPI paid plan | approved on the free plan only | Jim's check | 30 days |
| Claude Code extra usage | no agent changes the plan | **phase 2:** overage flag every triage run, plus Jim's check | three times a day, and 30 days |
| Any radar vendor not listed above (Squarespace, Auth0, RapidAPI, Claude Code, and each new one) | rule 1; a new vendor needs Jim's sign-off | The billing check's per-vendor list: an API check where one exists, else a settings check in `standards/settings.toml` | weekly or 30 days |
| A new vendor or `[[cost]]` | COST-004, the PR cost summary labels `needs-human` | the PR cost summary | every PR |

## Thresholds

One ladder for every limit, measured against our budget (or the free limit
where we have no budget):

| Level | Budget set | No budget | Meaning | What happens |
| --- | --- | --- | --- | --- |
| Green | below 80% of budget | below 50% of free | fine | nothing |
| Yellow | 80% of budget or more | 50% of free or more | watch | the usage run writes a warning, the digest lists it, pane shows yellow |
| Red | 100% of budget or more | 80% of free or more | stop growth | the usage run fails (repo health badge red), on-demand guards refuse to create more, triage stops whatever automation caused it and files a human issue |

Daily limits (Workers requests, D1 rows, KV) use the busiest day of the
last seven, because they reset at 00:00 UTC.

Money has no yellow. Any actual charge that isn't covered by an accepted
decision record is red at once, and any overrun switch found on is red at
once.

Claude Code: yellow when the settings check reports more than $20 spent in
the month; red when any session reports overage usage or the spend is over
$25.

## Where it shows up

| Where | What | Cadence | Status |
| --- | --- | --- | --- |
| Every PR | Cost summary job: expected usage and money that moved, `cost` and `needs-human` labels | every push | done (cost check phase 2) |
| Repo health badge "Secrets and free tiers" | red when a limit is red or a billing check fails | each usage run | done; phase 1 runs it daily |
| PR triage | reads the badge and fixes or reports red (repo-health.md); phase 2 adds the Claude overage check | three times a day | partly done |
| Weekly human issues digest | phase 2 adds a short **Cost** section: money this month, every yellow and red limit, any switch whose check is due | weekly | new |
| pane Money page | headroom table and money per project | on each pane build | planned (pane Phase 7 issues) |
| Monthly cost file | `standards/cost/usage/<yyyy-mm>.json`: live peak usage against expected and budget, for trends and the monthly review | monthly | planned (`cost/docs/issues/cost-check-phase-4-replaces-old.md`) |

So a problem reaches Jim through at most three channels, in this order: a
red badge (within a day), the triage report (within hours of the badge),
and the digest (weekly), with pane as the place to look anything up.

## What needs Jim

- **Answers now:** the list of Squarespace domains with their prices and
  renewal dates, and a Squarespace budget (phase 4,
  [runbook](../runbooks/squarespace-domains.md)).
- **Credentials, through the second approval (decision 0010):** phase 1
  needs the usage token to read Cloudflare account analytics; phase 3 needs
  an Auth0 token with `read:stats` and a number for
  `auth0_m2m_tokens_per_month`. Infra Mint makes both (decision 0011); Jim
  approves the run. No new vendor, no money.
- **Ongoing:** the 30-day settings checks he already does for Claude,
  Auth0 and RapidAPI billing, which have no API. Nothing new is added to
  his list by this plan.

## Phases

Each phase is an issue file with `epic: Cost tracking`.

### Phase 1: measure everything an API shows

- The usage run moves from weekly to daily (07:00 UTC), on its own workflow
  so the weekly full-history secret scan stays weekly. It writes
  `usage.json` as an artifact (shared with pane's issue
  `pane/docs/issues/give-free-tiers-sh-usage-json.md`), and applies the
  green/yellow/red ladder above, read from `limits.toml`.
- New live reads: repo visibility, Cron Triggers, D1 databases and size, R2
  buckets, and the busiest day's Workers requests and D1 rows from the
  Cloudflare GraphQL analytics API.
- `limits.toml` gains `warn_pct` and `stop_pct` only if a limit needs other
  levels than the ladder; otherwise the ladder is code.

### Phase 2: Claude Code and the digest

- PR triage reads `isUsingOverage` from the session API each run. If it is
  true, it reports red and the coordinator starts no new threads until the
  next reset (AGENTS.md, Jim, 2026-11-27).
- The `human-issues-digest` skill gets the Cost section.
- free-tiers.md gets an "Overrun switches" section with the table above,
  so a new vendor adds its switches in the same PR.

### Phase 3: Auth0

- A weekly read of Auth0 monthly active users (`GET /api/v2/stats/active-users`)
  and a count of our own M2M token requests (one per infra run), against
  the budget Jim sets.

### Phase 4: Squarespace domains

Jim has several paid domains on Squarespace, the one place we pay money
outside Claude Code. Squarespace has no public API for domains, so the
list comes from Jim once and is then kept true by a settings check.

- Jim lists each domain, its yearly price, renewal date, auto-renew
  setting and what it is for (human task, runbook
  [squarespace-domains.md](../runbooks/squarespace-domains.md)), and sets
  the Squarespace budget in the radar.
- Each domain becomes a `[[cost]]` (`every = "year"`, `vendor =
  "Squarespace"`) in the `docs/cost.toml` of the project it serves, or in
  `standards/docs/cost.toml` if it serves none, backed by one decision
  record for all of them.
- `[[cost]]` gains a `renews` date. The digest lists renewals due in the
  next 60 days, so Jim can drop a domain before it is charged again, and
  domains that serve no project are flagged as ones to consider dropping.
- The `squarespace-billing` settings check (every 90 days, from the
  billing check work) also confirms the list, prices and auto-renew still
  match the `[[cost]]` entries.

### Then

The monthly cost file, `cost render --ledger` and the `review-costs` skill
are already planned in `cost/docs/issues/cost-check-phase-4-replaces-old.md`; they
read phase 1's `usage.json` instead of a new source.

## Cost of this process

- **Money:** $0. Everything runs on GitHub Actions standard runners on a
  public repo and free Cloudflare and Auth0 APIs. No new vendor.
- **Free tiers:** a daily usage run is about 30 runner minutes a month
  (unlimited on a public repo). Cloudflare API and GraphQL reads are not
  metered against Workers. Auth0: about 5 M2M tokens a month for the weekly
  stats read, inside the budget Jim sets.
- **Claude Code:** about four agent sessions to build (one per phase plus
  review fixes), then a few extra minutes in each triage run and weekly
  digest. All of it counts against the $25 a month budget, so the phases
  are backlog work, not priority.

## Decisions for Jim

Recommended defaults, which agents follow unless Jim says otherwise:

1. **Thresholds:** the ladder above (yellow at 80% of budget, red at 100%).
2. **Usage run:** daily instead of weekly. $0; about 30 runner minutes a
   month on a public repo.
3. **Claude overage (decided, Jim, 2026-11-27: "Pause and tell me"):**
   when a session reports overage usage, PR triage reports it to Jim and
   the coordinator starts no new threads until the usage resets. $0; it can
   only lower Claude spend. It narrows the coordinator's "keep at least 5
   threads working" rule to "while not on overage", and is in AGENTS.md.
4. **Squarespace (Jim, 2026-11-27: "I have several domains in square
   space yes"):** each paid domain becomes a yearly `[[cost]]` with a
   decision record, so actual money is complete (phase 4).

## Not doing

- Reading vendor bills beyond what the billing checks already read. They
  are owned by the free-tier payment check work.
- Forecasting paid-plan prices. The rule is stop, don't pay.
- A database or a hosted dashboard. Results are JSON artifacts and files in
  the repo, shown by pane.
