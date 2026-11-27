# 0013: Start work by the Claude usage window, checked every hour

- Status: Accepted (Jim asked for it, 2026-11-27)
- Date: 2026-11-27
- Related: [cost tracking plan](../plans/2026-11-cost-tracking.md),
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
| A. Hourly Claude scheduled task that wakes the coordinator, which reads its own rate-limit events and starts threads by a rule (chosen) | $0 (part of the plan) | Uses the five-hour window up to about 95%, keeping the seven-day window on pace | One PR and one scheduled task |
| B. A fresh session every hour that reads the numbers and messages the coordinator | $0 | Same | Same, plus a session start every hour, which itself uses the window |
| C. GitHub Actions cron | $0 | n/a | Not possible: Actions can't read Claude's usage |
| D. Keep the flat 5 threads | $0 | Leaves part of most windows unused | None |

## Decision

Option A. A recurring Claude scheduled task, "Credit window check", fires
every hour into the coordinator's session. Its exact prompt is in
[scheduled-tasks/credit-window.md](../scheduled-tasks/credit-window.md).
Each run the coordinator:

1. Reads its newest `rate_limit_event`s and the open threads.
2. Applies the rule below, then starts that many new threads with the
   `start-a-new-task` skill, keeping the usual limits (one project per
   thread, at most 30 open, skip on-hold projects and mise migration files).
3. Says nothing in the project chat unless it started threads, paused, or
   hit a limit, so hourly runs don't flood Jim.

### The rule

Let `u5` and `u7` be the five-hour and seven-day utilization, `h` the
hours until the five-hour reset, `d` the days until the seven-day reset,
and `W` the threads working now (not waiting on Jim).

1. **Stop.** If `isUsingOverage` is true or any status is not `allowed`,
   start nothing and tell Jim (the overage rule in AGENTS.md wins).
2. **Burn rate.** `r` = the rise in `u5` per hour between the newest event
   and one at least 10 minutes older in the same window (same `resetsAt`).
   With no such pair, `r = 0.02 × W`. The cost of one more thread is
   `t = r / W`, or 0.02 an hour when `W` is 0.
3. **Projection.** `p = u5 + r × h`, the use expected when the window resets.
4. **Weekly guard.** The week's elapsed share is `e = 1 − d / 7`. If
   `u7 > e + 0.15` (ahead of an even pace through the week) and `d > 0.5`,
   don't burst: keep only the 5-thread floor, so the seven-day window
   doesn't run out days early.
5. **Burst.** Otherwise, if `p < 0.85`, start
   `n = ceil((0.95 − p) / (t × max(h, 0.5)))` threads, at most 8 a run and
   never more than 30 open.
6. **Ease off.** If `p > 1.0` and `h > 1`, start nothing beyond the
   5-thread floor; running threads finish their work.
7. **Floor.** In every other case keep the existing floor of 5 working
   threads.

Aiming at 95% rather than 100% leaves room for error: reaching the limit
early stalls every thread until the reset, which costs more than a few
unused points.

## Consequences

- Most five-hour windows end nearly used, and the seven-day window still
  lasts the week.
- The hourly wake itself uses a little of the window (one short turn).
- The coordinator's "at least 5 threads" rule becomes the floor of this
  rule rather than the whole rule (AGENTS.md "Credit window").
- The 0.02-per-thread default and the 0.85/0.95 thresholds are first
  guesses. Each run states the numbers it used in its turn, so they can be
  tuned from real windows in a later PR.
- If Claude stops recording `rate_limit_event`s, or renames their fields,
  the run falls back to the 5-thread floor and tells Jim once.
- The scheduled task is a setting made by hand, listed with the others in
  `standards/settings.toml` (`claude-scheduled-tasks`).
