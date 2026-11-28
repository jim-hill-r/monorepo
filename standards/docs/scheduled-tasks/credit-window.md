# Credit window check scheduled task

The prompt of the "Credit window check" scheduled task in the Claude
project: every hour at :53 UTC it starts a fresh run (a short project
thread) that reads the usage, counts the working threads, asks the
coordinator to start threads when the rule says so, and resolves its own
thread. No standing thread runs the process (Jim, 2026-11-28). Why and the
rule's reasoning are in
[decision 0013](../decisions/0013-credit-window-scheduler.md). When this
file changes, an agent recreates the scheduled task with the new prompt
once the PR merges (a task that starts fresh runs can't be edited in
place), and says so in its thread.

History:

- 2026-11-27: created (Jim: "kick off work as our credit window begins to
  close to maximize our use of the 5 hr credits. Have that process run all
  the time").
- 2026-11-27: moved from the coordinator's session to the credit window
  thread's session, after the first run had no tools to read usage or
  start threads. Verified the same day.
- 2026-11-27: every unresolved thread, blocked ones included, counts
  toward the 30 cap (the coordinator's count), and the thread list is
  paged to the end.
- 2026-11-27: no cap on open threads (Jim: "Lets remove the 30 caps in
  an effort to keep credit window maximized. I will catch up
  eventually").
- 2026-11-27: default per-thread cost 0.04 an hour, measured that day.
- 2026-11-27: the burn rate also uses the rate since the last burst, after
  8 new threads moved the window 16 points in 8 minutes.
- 2026-11-27: `allowed_warning` (the 90% notice) is not a stop; only
  overage or a `rejected` status is.
- 2026-11-28: a fresh run each hour instead of waking the standing credit
  window thread (Jim: "We shouldn't have a thread running this process.
  Can we use a schedule task instead?"). Each run resolves its own thread;
  the burn rate uses a pair of events 10 to 30 minutes apart, since a run
  can't remember the last one; easing off also skips floor threads.
  Verified the same day.

## Prompt

```text
Credit window check (hourly) for Jim's Claude project on the GitHub repo jimhillr/monorepo. Rule: standards/docs/decisions/0013-credit-window-scheduler.md; this prompt is kept in standards/docs/scheduled-tasks/credit-window.md on main. Jim, 2026-11-27: "kick off work as our credit window begins to close to maximize our use of the 5 hr credits. Have that process run all the time." Jim, 2026-11-28: run it as a scheduled task, not a standing thread. So this run is short and cheap (no repo clone, no code work, no checklist, no post to Jim unless step 6 says so), and it resolves its own thread at the end.

1. Read usage. Call list_events (claude-code-remote) on the coordinator session session_012EWF9eKKjMkhJ5U3b3BtdT with kinds ["rate_limit_event"] and limit 100, paging back with before_id if needed. Each event's rate_limit_info is account-wide: unifiedWindows.five_hour {utilization u5, resetsAt}, seven_day {utilization u7, resetsAt}, isUsingOverage, status. If the coordinator has no event in the last 30 minutes, also read your own session's events. Newest event gives u5, u7 and the resets. h = hours to the five-hour reset, d = days to the seven-day reset.
2. Burn rate r = the rise in u5 per hour between the newest event and the newest event at least 10 minutes older within the same five-hour resetsAt (prefer a pair 10 to 30 minutes apart, so a recent burst of new threads isn't hidden). No usable pair (fresh window): r = 0.04 * W.
3. Count W = threads in state working (list_threads with state "working", paging with next_cursor; don't count this thread or other "Credit window check" threads).
4. Rule. STOP if isUsingOverage is true or any status is "rejected" ("allowed_warning", the 90% notice, is not a stop). Projection p = u5 + r * h. Per-thread cost t = r / W, or 0.04 an hour when W is 0. Weekly guard: e = 1 - d/7; if u7 > e + 0.15 and d > 0.5, no burst (floor only). Burst if p < 0.85: n = ceil((0.95 - p) / (t * max(h, 0.5))), at most 8. HOLD if p > 1.0 and h > 1, or if u5 + r * (h - 1) > 1.0 (100% more than an hour before reset): n = 0, and no floor threads either. Otherwise floor: n = max(0, 5 - W).
5. Act. Only when n > 0, HOLD or STOP, send ONE message_coordinator starting "Credit window check (scheduled task, decision 0013):", naming the action, n, the numbers (u5, h, r, p, u7, d, W) and the projects that already have working threads. Burst or floor: "start n threads with start-a-new-task, one project each, skipping luggage, starcraft, profiles and mise migration files". HOLD: "start no new threads until the reset at <time UTC>". STOP: "overage or a rejected status: start nothing and tell Jim". The coordinator starts the threads. Otherwise send nothing.
6. Only if usage can't be read (no rate_limit_event, renamed fields, missing tools), or on STOP, post_message one line for Jim saying what failed. Then call set_thread_resolved with resolved true, and end the turn with no_reply_needed.
```
