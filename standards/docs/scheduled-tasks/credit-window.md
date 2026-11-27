# Credit window check scheduled task

The prompt of the "Credit window check" scheduled task in the Claude
project: every hour, fired into the session of the standing credit window
thread ("Use up credits before each 5-hour window closes"), which reads
the usage and asks the coordinator to start threads. A task fired at the
coordinator's own session doesn't work (it runs as a separate routine
session without the session-API or thread tools). Why and the rule's
reasoning are in
[decision 0013](../decisions/0013-credit-window-scheduler.md). When this
file changes, an agent updates the scheduled task's prompt to match once
the PR merges, and says so in its thread.

History:

- 2026-11-27: created (Jim: "kick off work as our credit window begins to
  close to maximize our use of the 5 hr credits. Have that process run all
  the time").
- 2026-11-27: moved from the coordinator's session to the credit window
  thread's session, after the first run had no tools to read usage or
  start threads. Verified the same day.

## Prompt

```text
Credit window check (hourly), decision 0013. You are the credit-window thread; this wakes you each hour. Follow standards/docs/scheduled-tasks/credit-window.md on main if it exists, else the version on branch claude/project-thread-tyvz8g.

1. Read usage: list_events on the coordinator session session_012EWF9eKKjMkhJ5U3b3BtdT and on your own session, kinds ["rate_limit_event"], limit 100. Newest event gives u5, five-hour resetsAt, u7, seven-day resetsAt, isUsingOverage, status. Burn rate r from the newest event and the newest one at least 10 minutes older with the same five-hour resetsAt (page back with before_id if needed).
2. Count threads with list_threads: W = working, O = open (not done/failed/stopped).
3. Apply the rule (STOP, weekly guard, burst to 95%, ease off, 5-thread floor) and get n.
4. If n > 0, or STOP applies, send the coordinator one message_coordinator naming n, the numbers, and the projects that already have open threads. The coordinator starts the threads (start-a-new-task, one project each, skip luggage/starcraft/profiles and mise migration files, max 30 open). Otherwise send nothing.
5. Update this thread's checklist with one line: time, u5, h, r, p, u7, W, O, n. Don't post_message unless something needs Jim (overage, unreadable usage, 30 threads). End the turn with no_reply_needed otherwise.
```
