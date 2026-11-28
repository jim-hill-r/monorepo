# 0013: Start work by the Claude usage window, checked every hour

- Status: Accepted (Jim asked for it, 2026-11-27)
- Date: 2026-11-27
- Related: [cost tracking plan](../../../cost/docs/plans/2026-11-cost-tracking.md),
  [scheduled task prompt](../scheduled-tasks/credit-window.md)

## Context

Jim, 2026-11-27: "let's establish a process based on credit usage where we
kick off work as our credit window begins to close to maximize our use of
the 5 hr credits. Have that process run all the time."

Claude Code on Jim's plan has a rolling five-hour usage window and a
seven-day window. Whatever is left of a five-hour window when it resets is
lost. Until now the coordinator kept a flat "at least 5 threads working",
which leaves part of most windows unused, and on 2026-11-26 the opposite
happened: about ten sessions hit the five-hour limit around 13:40 UTC and
waited until it reset at 17:40 UTC.

### What we can read

Every Claude Code session records a `rate_limit_event` after its API
calls. The session API returns them (`list_events` with
`kinds: ["rate_limit_event"]`), and they hold account-wide numbers, so any
session's latest event describes the whole account. A real event from this
thread:

```json
"rate_limit_info": {
  "isUsingOverage": false,
  "overageStatus": "rejected",
  "overageDisabledReason": "org_level_disabled_until",
  "status": "allowed",
  "unifiedWindows": {
    "five_hour": {"resetsAt": 1795798800, "status": "allowed", "utilization": 0.45},
    "seven_day": {"resetsAt": 1795842000, "status": "allowed", "utilization": 0.45}
  }
}
```

So the five-hour window's use (0 to 1) and its reset time, and the same
for the seven-day window, can be read automatically. Overage is disabled
at the organization level, so using the window fully costs no money.
Dollars are still not readable.

What we can't read: there is no Claude usage API a GitHub workflow could
call, and no event that fires as a window nears its end. Only a Claude
session can read these numbers, and only the coordinator (the project's
own session) can start threads.

### Options

| Option | Money | Free-tier effect | Build effort |
| --- | --- | --- | --- |
| A. Hourly Claude scheduled task that wakes a standing "credit window" thread, which reads the rate-limit events and asks the coordinator to start threads (chosen first, replaced by E on 2026-11-28) | $0 (part of the plan) | Uses the five-hour window up to about 95%, keeping the seven-day window on pace | One PR and one scheduled task |
| B. Hourly scheduled task fired at the coordinator's session | $0 | Same | Tried first and failed: the run arrives as a separate routine session with no session-API or thread tools, so it can neither read usage nor start threads |
| C. GitHub Actions cron | $0 | n/a | Not possible: Actions can't read Claude's usage |
| D. Keep the flat 5 threads | $0 | Leaves part of most windows unused | None |
| E. Hourly Claude scheduled task that starts a fresh run each hour, which reads the usage, counts the threads, messages the coordinator and resolves itself (chosen, Jim 2026-11-28: "We shouldn't have a thread running this process. Can we use a schedule task instead?") | $0 | Same as A | One PR and one scheduled task; each run reloads the project context, a little more per run than A |

## Decision

Option E. A recurring Claude scheduled task, "Credit window check", runs
every hour at :53 UTC and starts a fresh run each time. A fresh run is a
short project thread with the project's own tools: the session API
(`list_events`) to read usage, `list_threads` to count threads, and
`message_coordinator`. That was verified on 2026-11-28; the first attempt
(option B) failed because it was bound to the coordinator's session.
Option A worked but kept a standing thread open all the time, which Jim
didn't want (2026-11-28). Its exact prompt is in
[scheduled-tasks/credit-window.md](../scheduled-tasks/credit-window.md).
Each run:

1. Reads the newest `rate_limit_event`s of the coordinator's session (its
   own when the coordinator has none recent), and lists the project's
   working threads.
2. Applies the rule below.
3. When the rule asks for threads (or says hold or stop), sends the
   coordinator one message with the number and the numbers behind it.
   The coordinator, the only session that can start threads, starts them
   with the `start-a-new-task` skill under the usual limits (one project
   per thread where possible, skip on-hold projects and mise migration
   files; no cap on open threads, Jim 2026-11-27: "Lets remove the 30
   caps in an effort to keep credit window maximized. I will catch up
   eventually"). It picks the issues with `chores next-task --count n
   --working <the working threads' projects>`, which keeps at least half
   of the work on products (Jim, 2026-11-28, see "Product share" below).
   Otherwise it sends nothing, so hourly runs don't flood anyone.
4. Resolves its own thread, so no thread stays open for the process. It
   posts to Jim only when usage can't be read or on a stop.

A fresh run has no memory of the last one, so the burn rate comes from a
pair of rate-limit events 10 to 30 minutes apart, which still shows a
recent burst of new threads.

### The rule

Let `u5` and `u7` be the five-hour and seven-day utilization, `h` the
hours until the five-hour reset, `d` the days until the seven-day reset,
`W` the threads working now (not waiting on Jim), and `O` the open
threads (every thread not yet resolved), which is reported but caps
nothing.

1. **Stop.** If `isUsingOverage` is true or any status is `rejected`
   (`allowed_warning`, the 90% notice, is not a stop),
   start nothing and tell Jim (the overage rule in AGENTS.md wins).
2. **Burn rate.** `r` = the rise in `u5` per hour between the newest
   event and the newest event at least 10 minutes older (a pair 10 to 30
   minutes apart where there is one), in the same window (same `resetsAt`).
   Freshly started threads burn fastest (on 2026-11-27, 8 new threads
   took the window from 3% to 19% in 8 minutes), so an hour-old sample
   must not hide a recent burst. With no usable pair, `r = 0.04 × W`. The
   cost of one more thread is `t = r / W`, or 0.04 an hour when `W` is 0
   (measured on 2026-11-27: a working thread uses about 4% of a window
   per hour, so about 5 working threads fill one).
3. **Projection.** `p = u5 + r × h`, the use expected when the window resets.
4. **Weekly guard.** The week's elapsed share is `e = 1 − d / 7`. If
   `u7 > e + 0.15` (ahead of an even pace through the week) and `d > 0.5`,
   don't burst: keep only the 5-thread floor, so the seven-day window
   doesn't run out days early.
5. **Burst.** Otherwise, if `p < 0.85`, start
   `n = ceil((0.95 − p) / (t × max(h, 0.5)))` threads, at most 8 a run. There is
   no cap on open threads: threads waiting on Jim don't use credits, so
   they don't hold back new work.
6. **Ease off.** If `p > 1.0` and `h > 1`, or the window would reach
   100% more than an hour before reset, start nothing, not even floor
   threads; running threads finish their work. This holds even
   when a longer average looks lower: reaching the limit early stalls
   every thread until the reset.
7. **Floor.** In every other case keep the existing floor of 5 working
   threads.

Aiming at 95% rather than 100% leaves room for error: reaching the limit
early stalls every thread until the reset, which costs more than a few
unused points.

## Consequences

- Most five-hour windows end nearly used, and the seven-day window still
  lasts the week.
- The hourly wake itself uses a little of the window (one short turn).
- Each run is its own short thread, resolved at the end, so nothing
  depends on one long-lived session. The weekly settings check
  `claude-scheduled-tasks` catches a task that stopped running.
- The coordinator's "at least 5 threads" rule becomes the floor of this
  rule rather than the whole rule (AGENTS.md "Credit window").
- The 0.04-per-thread default and the 0.85/0.95 thresholds are first
  guesses. Each run states the numbers it used in its turn, so they can be
  tuned from real windows in a later PR.
- If Claude stops recording `rate_limit_event`s, or renames their fields,
  the run falls back to the 5-thread floor and tells Jim once.
- The scheduled task is a setting made by hand, listed with the others in
  `standards/settings.toml` (`claude-scheduled-tasks`).

## Product share (2026-11-28)

Jim, 2026-11-28: "I'd like to ensure that at least 50% of all of my claude
credits are being used on product work, not to let the platform work
overwhelm it. Try to include some changes into the start a task which
chooses tasks for this very soft heuristic. The goals being equal effort on
platform stuff and product stuff."

Credits aren't readable per thread, so `chores next-task` estimates the
split from two signals, each read by tier (decision 0019; the
`software-development-platform` and `technical-platform` tiers count as
platform, root `issues/` work too):

- the threads working now, which the coordinator passes as `--working`,
  one project per thread, and
- the commits on `main` in the last seven days (`--days`, one squash
  merge per PR), each on the side most of its files are in.

The product share is the mean of the two shares. While it is under 50%,
each pick is the first qualifying product issue in the usual order
instead of the first issue, and each pick counts as a new product or
platform thread, so a burst of eight picks evens out rather than going
all one way. At 50% or more, or when nothing can be counted, the usual
order applies. It is soft on purpose: priority order still decides within
each side, `--project` picks skip it, and with no product issue left a
platform issue is picked. `--no-balance` turns it off.

A project's tier is read from its own `docs/project.toml` (`tier`), which
[decision 0022](0022-project-metadata-and-product-docs.md) proposes; until
those files exist, `chores` falls back to 0019's table, and phase 1 of the
product strategy epic deletes that copy (Jim, 2026-11-28: "we want every
project to be very independent. No centralized directories").
