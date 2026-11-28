# 0032. Performance testing: lab load on a local Worker, budgets as evals, production only watched

- Status: Proposed
- Decided by: Jim (on merging the pull request that adds this record)
- Date: 2026-11-28
- Asked by: Jim ("We are starting to add a lot of really good features to
  our products. How are we handling performance testing? We need to add a
  tech solution for that? This is related to e2e testing and chaos testing
  mentioned in other projects.")

## Context

The standard these tools enforce is [performance.md](../performance.md).
This record is about the tools and where they run.

### What we test today

| Layer | Covered by | Gap |
| --- | --- | --- |
| Page loads on our sites | Lighthouse CI and Core Web Vitals (decision 0028, [web.md](../web.md) WEB-3xx), being rolled out | None for loads; nothing times what happens after the page is up (search, saving, a game turn) |
| User flows | Playwright e2e and `@smoke` tests (decision 0005, [testing.md](../testing.md)) | They check that a flow works, never how long it takes |
| Faults | The eval plan's chaos stage, with `slow-3g` and `api-slow` faults ([eval engine plan](../plans/2026-11-eval-engine.md#chaos-testing-in-production)), not built yet | Single-user by design: the plan says "no load testing, ever" against production |
| Our Workers and D1 | `tofu_state`'s end-to-end test against `wrangler dev` with a local D1; `http` evals can carry a `max_ms` | Nothing measures latency under concurrency, CPU time against the 10 ms free limit, or D1 rows read per request |
| Hot library code and games | Nothing (royalty's counter plans a 2 ns benchmark gate) | No benchmarks, no frame-time budget |

We run three Workers today (`jimhillr-pane`, `jimhillr-telemetry`,
`jimhillr-tofu-state`, the last on D1), and decisions 0018, 0020, 0021 and
0023 add a flags Worker, feedback endpoints, a shared product D1 and a
watchdog. So the gap that grows fastest is the backend.

### What a slow backend costs us here

On the Workers Free plan a performance problem is an **outage, not a
bill** ([free-tiers.md](../free-tiers.md#cloudflare-pages-workers-d1-r2)):

- a request over **10 ms of CPU** fails with error 1102;
- past **100,000 requests a day** every Worker on the account returns error
  1027 until 00:00 UTC;
- past **5,000,000 D1 rows read** or **100,000 rows written a day**, D1
  queries fail. One missing index turns a 10-row lookup into a full-table
  scan, which is the easiest way for one feature to take down every product
  on the shared database (decision 0021).

So "fast enough" here means three things: quick for the person, under the
CPU limit with room to spare, and cheap in rows and requests so the account's
daily quotas last.

### What a tool must do

- **$0**, no new vendor, run in GitHub Actions on a standard runner (FT-001),
  no local clone.
- **Never load production.** Load against a Cloudflare URL spends the
  account-wide quota every product shares, and the eval plan's chaos rule
  forbids it. So load must run against a Worker on the runner itself.
- **Rust and TOML, not a new scripting language.** The repo's code is Rust
  ([README](../README.md#languages)); scripts are Rust-only (decision 0015).
- Results land in the eval engine, so a budget is an eval that must stay
  green and a red one becomes an issue file (AGENTS.md, "Evals are
  continuous improvement").

## Options

For the load generator (the part that is new):

| | Money | Language of a test | Licence | Fits | Notes |
| --- | --- | --- | --- | --- | --- |
| **A. oha, driven by TOML in `evals`** | $0 | TOML budgets in `evals.toml`; no script | MIT | Rust CLI, one binary; fixed rate (`-q`), duration (`-z`), connections (`-c`), method, headers and body; JSON summary with latency percentiles, status counts and errors | One endpoint per run, no multi-step flows; our Workers' calls are single requests, so a scenario is a list of endpoints. The `evals` CLI runs it and checks the budgets |
| B. k6 | $0 (open source; Grafana Cloud k6 not used) | JavaScript | AGPL-3.0 | Best known; scenarios, arrival-rate executors, thresholds that fail the run | Brings JavaScript test code into the repo, against the languages rule; its thresholds would duplicate the eval engine's budgets |
| C. Goose | $0 | Rust (each test is a compiled binary) | Apache-2.0 | Rust-native scenarios and weighted tasks | Heavy build per test project; less known to Claude; more than single-request Workers need |
| D. Artillery, Locust, autocannon | $0 | YAML+JS, Python, JS | Mixed | Popular | New language runtimes for one job |
| E. Cloudflare-hosted load (Browser Run, a load Worker) or a SaaS (Grafana Cloud k6, BlazeMeter) | Free tiers too small or a new vendor | - | - | - | Loads production, so ruled out by the rule above |

For where load runs:

| | Spends Cloudflare quota | Measures | Notes |
| --- | --- | --- | --- |
| **Local `wrangler dev --local` on the runner, local D1 seeded with realistic data** | None | Latency under concurrency, errors, D1 rows read and written per request, timeouts and retries under injected faults | `tofu_state/tests/e2e.sh` already does this; runner numbers are lab numbers, so budgets are regression budgets, not promises about the edge |
| A preview deploy | Yes (same account) | Edge latency | Same quota as production; ruled out |
| Production | Yes | Real latency | Ruled out; production is watched instead (below) |

## Decision

Performance testing is four layers, and only one of them is new tooling:

1. **Page loads: Lighthouse CI**, as decision 0028 decided. Nothing changes.
2. **Backend load, in the lab: option A.** A new `load` eval kind in the
   `evals` CLI reads budgets from the project's `evals.toml`, starts nothing
   itself, and runs a pinned **oha** against `BASE_URL`. A shared mise task,
   `mise run perf`, starts `wrangler dev --local` with a local D1 seeded from
   the project's fixtures, runs `evals run --stage ci` for the `load` evals
   and stops the Worker. It runs in PR CI for each Worker a PR changes, and
   weekly on `main` for every Worker, never against a Cloudflare URL. A
   Worker reports the D1 rows each request read and wrote in a
   `Server-Timing` header only when the local-only `JIMHILLR_PERF` variable
   is set, so the eval can hold a rows-per-request budget, the check that
   catches a missing index before it reaches the shared database.
3. **User flows: timed Playwright tests.** Key interactions (a search, a
   save, a game turn) get a `@perf` Playwright test that times the flow
   with `performance.mark` and `measure` and asserts the requirement's time,
   on the build in CI and as a single-user `@smoke` run after deploy. They
   are ordinary `playwright` evals, so no new tool.
4. **Production: watched, never loaded.** A few `http` evals with `max_ms`
   after each deploy (a capped number of requests), and `metric` evals in
   the `observe` stage that read the Workers and D1 analytics Cloudflare
   already collects (CPU time p99, wall time p95, error rate, D1 rows a
   day) through the GraphQL Analytics API with a read-only token. Reading
   analytics spends no Workers requests.

Chaos ties in where speed meets failure: the eval plan's `slow-3g` and
`api-slow` faults check that pages stay usable when things are slow (one
user, in production), and the lab load run adds the same faults under load
on the local Worker, so we see whether a slow upstream makes our Worker
time out cleanly or pile up retries. The plan's rule stands: chaos in
production is never load.

Games and hot library code get budgets in their own scripted runs and
benchmarks (PERF-5xx); the benchmark tool is assessed in its own issue,
gating on instruction counts rather than wall time, since shared runners
are too noisy for nanosecond timing.

Rejected: B brings JavaScript tests and a second threshold system; C costs
build time for no gain on single-request endpoints; D adds runtimes; E and
preview deploys spend the quota every product shares. k6 stays on the radar
in Hold, as the fallback if a Worker ever needs a scripted multi-step flow
oha can't express.

## Consequences

- **Cost:** $0 at every scale below. No new vendor, no paid plan, no
  Cloudflare usage from load. Each lab run is about 3 minutes of standard
  runner time (start the Worker, seed, three 30-second steps), free on a
  public repo. The one new credential is a read-only Account Analytics
  token, made by Infra Mint (decision 0011) in phase 4, with a row in
  [secret-rotation.md](../secret-rotation.md).

  | | Workers (projects) | Lab runner time a month | Weekly full run, wall time | Production requests a day | Analytics queries a day |
  | --- | --- | --- | --- | --- | --- |
  | Today | 3 (29) | about 160 minutes | 3 minutes | about 15 (0.015% of the free 100,000) | about 10 (GraphQL allows 300 per 5 minutes) |
  | 5x | 15 (145) | about 780 minutes | 3 minutes (15 jobs in parallel) | about 75 (0.08%) | about 45 |
  | 100x | 300 (2,900) | about 15,600 minutes | about 45 minutes (20 concurrent jobs) | 500, the cap (0.5%); probes rotate across Workers | about 900 |

  Assumes three PR runs and one weekly run per Worker a week, and at most
  five `max_ms` probes per Worker per deploy, under a hard cap of 500
  production probe requests a day across the account. At 100x the
  binding limits are elsewhere: the account allows 100 Workers (our budget
  is 25), so a much larger product count needs a decision record about
  accounts long before performance testing costs anything. If the repo
  ever goes private, the runner minutes would exceed the 2,000 free minutes
  a month at about 12x today; the weekly full run would then drop to
  changed Workers only.
- **Claude Code effort**, against the $25 a month budget: this record and
  the standard now (one thread); then about five threads, one per rollout
  phase in [performance.md](../performance.md#rollout). Each adoption by a
  further Worker is a small part of that project's own work.
- **Lab numbers are not edge numbers.** A runner is not Cloudflare's edge,
  and `wrangler dev` doesn't enforce the 10 ms CPU limit. Lab budgets catch
  regressions (a slower query, more rows, a retry loop); the CPU limit and
  real latency are watched from production analytics.
- **New budgets warn for a week first** and existing endpoints start from a
  measured baseline that only tightens, the same ratchet as decisions 0015
  and 0028, so a noisy runner doesn't block anyone.
- **The eval plan's "no load testing, ever"** now reads as "no load testing
  against anything Cloudflare runs": load moves to the runner, and the
  plan's production safety rules are unchanged.
- The rollout issues are listed in [performance.md](../performance.md#rollout).
