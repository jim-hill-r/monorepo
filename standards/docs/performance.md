# Performance

How fast our products must be, and how we test it, from the first paint to
the database. The tools and why we chose them are in
[decision 0032](decisions/0032-performance-testing.md).

Page loads are covered by [web.md](web.md) (WEB-3xx, Lighthouse CI and
Core Web Vitals); this standard covers everything else and how the pieces
fit together. Each budget is an eval tied to a requirement, so it shows in
the same verdicts as any other promise and must stay green
([evals.md](evals.md#continuous-improvement)).

Every rule has a stable ID and names its **enforcer**:

- **Load eval**: a `load` eval in `evals.toml`, run by `mise run perf`
  against the Worker on the runner (`wrangler dev --local`), never against
  Cloudflare.
- **Playwright**: a `@perf` Playwright test, run as a `playwright` eval.
- **Probe**: an `http` eval with `max_ms` in the `cd` stage.
- **Metric**: a `metric` eval in the `observe` stage, reading Cloudflare's
  Workers and D1 analytics.
- **Audit**: a check in `mise run //:audit`.
- **Review**: the PR reviewer and the `code-review` rubric.

Until a check lands (see [Rollout](#rollout)), review enforces its rule.

## How it fits together

| Question | Where it is answered | Rules |
| --- | --- | --- |
| Does the page load fast? | Lighthouse CI on the build and the live site; real users' Core Web Vitals | WEB-301 to WEB-308 |
| Does a flow feel fast once the page is up? | Timed Playwright tests, in CI and after deploy | PERF-3xx |
| Does our Worker stay fast and cheap under load? | Load evals against a local Worker with a seeded local D1 | PERF-1xx |
| Is production fast and under the free limits? | Probes after deploy, analytics metrics daily | PERF-2xx |
| Does it stay usable when things are slow or broken? | Chaos faults (`slow-3g`, `api-slow`) in production for one user, and under load in the lab | PERF-4xx |
| Are hot code and games within budget? | Scripted game runs and benchmarks | PERF-5xx |
| How do budgets change? | The process rules | PERF-6xx |

## Summary

| ID | Rule | Enforcer |
| --- | --- | --- |
| PERF-101 | Every Worker endpoint has a load eval | Audit (with a shrinking baseline) |
| PERF-102 | Load runs on the runner, never on Cloudflare | Load eval, audit |
| PERF-103 | Latency budgets under load: p95 and p99 | Load eval |
| PERF-104 | No errors under load | Load eval |
| PERF-105 | D1 rows read and written per request have a budget | Load eval |
| PERF-106 | The local D1 is seeded at realistic size | Load eval, review |
| PERF-107 | Subrequests per request have a budget | Load eval, review |
| PERF-201 | Production is never load tested | Audit, review |
| PERF-202 | Latency probes after each deploy, capped | Probe |
| PERF-203 | CPU time p99 at most half the free limit | Metric |
| PERF-204 | Production latency and error rate per Worker | Metric |
| PERF-205 | Account quotas keep headroom | Metric |
| PERF-301 | Key interactions are timed in Playwright | Playwright |
| PERF-302 | Timed flows also run on a slow phone | Playwright |
| PERF-401 | Slow upstreams time out within a budget | Load eval with a fault |
| PERF-402 | No retry storms | Load eval with a fault |
| PERF-403 | Readable when slow, for one user in production | Chaos (`fault` eval) |
| PERF-501 | Games hold a frame-time budget | `script` eval |
| PERF-502 | Hot library code gates on instruction counts | Benchmark in CI |
| PERF-601 | Budgets are evals tied to requirements | Audit (`evals check`) |
| PERF-602 | A baseline that only tightens | Load eval, review |
| PERF-603 | New budgets warn for a week first | CI |
| PERF-604 | Red results become issue files | Eval triage |
| PERF-605 | Expected usage uses measured costs | Review, `cost check` |

## PERF-1xx: backend load, in the lab

- **PERF-101: every Worker endpoint has a load eval.** Each route a Worker
  serves has at least one `load` eval in the project's `evals.toml`. The
  audit warns about a Worker (a `wrangler.toml` with `main`) whose routes
  have none, unless the project is in the baseline `standards/perf.toml`,
  which may only shrink.
- **PERF-102: load runs on the runner, never on Cloudflare.** `mise run
  perf` starts the built Worker with `wrangler dev --local` on `127.0.0.1`,
  with a local D1 and local bindings, and the `load` eval refuses any
  `BASE_URL` that isn't a loopback address. Nothing a load run sends reaches
  Cloudflare, so it spends none of the account's daily requests or D1 rows.
  It runs in PR CI for each Worker the PR changes and weekly on `main` for
  every Worker.
- **PERF-103: latency budgets under load.** Each `load` eval sets `p95_ms`
  and `p99_ms` at its `rate` (requests a second, held for `duration`,
  default 30 seconds). The default profile is three steps: 1 request a
  second (the cold, single-user time), 20, and 50. These are lab numbers on
  a shared runner, so they guard against regressions; they are not edge
  latencies.
- **PERF-104: no errors under load.** Every response is the expected status
  (`status`, default 200) and `max_error_rate` defaults to 0. A Worker that
  answers 429 by design (`telemetry`'s rate limiter) says so with
  `status = [200, 429]`.
- **PERF-105: D1 rows per request have a budget.** A Worker that uses D1
  sums `rows_read` and `rows_written` from each query's result metadata and,
  only when the local-only variable `JIMHILLR_PERF` is set, returns them as
  `Server-Timing: d1;desc="read=<n> written=<n>"`. The `load` eval sends one
  probe before the load and fails when `max_d1_rows_read` or
  `max_d1_rows_written` is exceeded. A query that should use an index and
  doesn't shows here as thousands of rows, long before it uses up the shared
  database's 5,000,000 rows a day (decision 0021). Production never sets the
  variable.
- **PERF-106: the local D1 is seeded at realistic size.** The run seeds the
  local D1 from the project's `perf/seed.sql` (committed when under 1 MB,
  otherwise generated by a test helper), sized at least as large as
  production is expected to be in a year, from `docs/cost.toml`. An empty
  table makes every query look fast.
- **PERF-107: subrequests per request have a budget.** A Worker that calls
  other services counts its `fetch` calls per request in the same
  `Server-Timing` header (`sub;desc="<n>"`), held by `max_subrequests`.
  During a load run, upstreams are local stubs, never the real GitHub,
  Auth0 or PostHog.

A `load` eval looks like this:

```toml
[[eval]]
id = "get-state-under-load"
requirement = "TOFU-004"
stages = ["ci"]
kind = "load"
method = "GET"
path = "/states/demo"
rate = [1, 20, 50]
duration = "30s"
p95_ms = 40
p99_ms = 80
max_d1_rows_read = 5
max_d1_rows_written = 0
```

`load` evals only ever have the `ci` stage.

## PERF-2xx: production, watched and never loaded

- **PERF-201: production is never load tested.** No workflow, eval or agent
  sends more than single-user traffic to anything Cloudflare runs for us,
  including preview deploys, which share the account's quota. This restates
  the eval plan's chaos rule ("no load testing, ever") for every stage.
- **PERF-202: latency probes after each deploy, capped.** Up to five `http`
  evals with `max_ms` per Worker in the `cd` stage, on the routes users hit
  most. All probes together send at most 500 requests a day across the
  account (0.5% of the free 100,000); the runner counts and stops at the cap,
  rotating which Workers it probes. Probes carry the `jimhillr-evals` user
  agent and `x-jimhillr-synthetic: 1`, so they are excluded from metrics.
- **PERF-203: CPU time p99 at most half the free limit.** Each Worker has a
  `metric` eval, source `workers-analytics`, `cpu_time_p99_ms` at most 5 over
  `7d`. The free limit is 10 ms a request (error 1102); half leaves room for
  a heavier request and a slower day.
- **PERF-204: production latency and error rate per Worker.** `metric` evals
  for wall time p95 (`duration_p95_ms`, the requirement's time) and the error
  share (`error_rate`, at most 0.5% over `7d`), from the same source.
- **PERF-205: account quotas keep headroom.** The account-wide daily Workers
  requests and D1 rows read and written, at their highest day over `7d`,
  stay under 50% of the free limit (an at-risk warning) and under 80% (a
  fail). These are `metric` evals in the `jimhillr` project, the same source,
  and the numbers come from [`limits.toml`](../radar/limits.toml).

The `workers-analytics` source reads Cloudflare's GraphQL Analytics API
(Workers invocations and D1 analytics) with a read-only Account Analytics
token, so it spends no Workers requests.

## PERF-3xx: user flows, in Playwright

- **PERF-301: key interactions are timed.** A requirement with a time in it
  ("search results show within 300 ms of typing", "a saved recipe is in the
  list within 1 s") has a Playwright test tagged `@perf` and
  `@req:<ID>`. The app marks the start and end with `performance.mark`, the
  test reads `performance.measure` through `page.evaluate` and asserts the
  time. It runs against the build in CI and, tagged `@smoke` too, as a
  single-user run after deploy. Lighthouse covers the page load; these cover
  what happens after it.
- **PERF-302: timed flows also run on a slow phone.** Each `@perf` test
  runs a second time with 4x CPU throttling (CDP
  `Emulation.setCPUThrottlingRate`) and a slow network, with the
  requirement's slow-phone time (by default twice the normal one). This is
  the same throttling as the `slow-3g` chaos fault, so a flow is timed the
  same way in CI and in chaos.

## PERF-4xx: speed under failure (chaos)

- **PERF-401: slow upstreams time out within a budget.** A Worker that calls
  another service sets a timeout below its own latency budget. A `load`
  eval with `fault = "api-slow"` or `"upstream-down"` (the eval plan's fault
  catalogue, injected into the local stubs) holds `p99_ms` at the timeout
  plus a margin, and expects the Worker's degraded answer (pane's "live data
  unavailable" and the snapshot), not a 5xx.
- **PERF-402: no retry storms.** Under the same faults, `max_subrequests`
  holds: a Worker retries an upstream at most once per request, with backoff,
  so a slow upstream never multiplies our requests or theirs.
- **PERF-403: readable when slow, for one user in production.** The
  `slow-3g` and `api-slow` `fault` evals in the `chaos` stage check the
  requirement's slow-phone time against production, one browser at a time,
  under the eval plan's request cap.

## PERF-5xx: games and hot code

- **PERF-501: games hold a frame-time budget.** A game's scripted run (the
  `script` eval kind) records frame times and holds a p95 budget against its
  own baseline on the runner (16.7 ms is the 60 fps target on real
  hardware; runners have no GPU, so the CI number is a regression budget).
- **PERF-502: hot library code gates on instruction counts.** Code with a
  stated cost (royalty's counter: "at most 2 ns per call") has a benchmark.
  The CI gate counts instructions (Valgrind-based), not wall time, because
  shared runners are too noisy for nanoseconds; wall-time benchmarks stay
  local and advisory. The tool is chosen in its rollout issue.

## PERF-6xx: process

- **PERF-601: budgets are evals tied to requirements.** Each Worker's
  `docs/requirements.md` has a requirement that it responds within its
  budget and stays within the free limits, and the PERF evals name it.
  `evals check` reports an eval with no requirement (EVL-001).
- **PERF-602: a baseline that only tightens.** An endpoint that misses a
  budget today records the measured value as its budget, with a `FIX` to
  bring it down. A PR that loosens a budget says why under `## Details`, and
  a budget only goes down otherwise, the same ratchet as WEB-403 and
  QUAL-501.
- **PERF-603: new budgets warn for a week first.** A new `load`, `@perf` or
  `metric` budget reports as a warning for at least a week of runs, then
  fails, so a noisy runner is caught before it blocks anyone (WEB-404).
- **PERF-604: red results become issue files.** A failing budget turns its
  eval red and eval triage writes a `FIX` in the project's `issues/`, the
  normal loop ([evals.md](evals.md#continuous-improvement)).
- **PERF-605: expected usage uses measured costs.** A Worker's
  `docs/cost.toml` multiplies its expected requests by the D1 rows and
  subrequests per request its load evals measured, so the cost check sees
  what a feature really spends against the free limits ([cost.md](cost.md)).

## Rollout

The checks land in phases, each an issue file:

1. `issues/perf-phase-1-load-eval.md`: the `load` eval kind in `evals`
   (oha, pinned in mise), the shared `perf` task and the loopback guard
   (PERF-102 to PERF-104), with `tofu_state` as the first Worker.
2. `issues/perf-phase-2-d1-rows.md`: the `Server-Timing` rows and
   subrequest counts behind `JIMHILLR_PERF` in a shared helper, seeded
   fixtures, and adoption by `telemetry` and `pane` (PERF-105 to PERF-107).
3. `issues/perf-phase-3-flows.md`: `@perf` Playwright tests and slow-phone
   runs, starting with the cookbook's search (PERF-301, PERF-302).
4. `issues/perf-phase-4-production.md`: the read-only analytics token
   through Infra Mint, the `workers-analytics` source and the probe cap
   (PERF-202 to PERF-205), after eval engine phase 3 adds `metric` evals.
5. `issues/perf-phase-5-faults-and-baseline.md`: faults in the lab load run
   (PERF-401, PERF-402), the PERF-101 audit with `standards/perf.toml`,
   and the game and benchmark budgets (PERF-501, PERF-502).
