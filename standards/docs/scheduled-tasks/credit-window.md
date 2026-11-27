# Credit window check scheduled task

The prompt of the "Credit window check" scheduled task in the Claude
project: every hour, fired into the coordinator's own session (the
project's ambient session), so the coordinator that starts threads is the
one that reads the usage numbers. Why and the rule's reasoning are in
[decision 0013](../decisions/0013-credit-window-scheduler.md). When this
file changes, an agent updates the scheduled task's prompt to match once
the PR merges, and says so in its thread.

History:

- 2026-11-27: created (Jim: "kick off work as our credit window begins to
  close to maximize our use of the 5 hr credits. Have that process run all
  the time").

## Prompt

```text
Credit window check (hourly). Jim asked on 2026-11-27 for a process that runs all the time and starts more work as the Claude five-hour usage window nears its end, so unused credits aren't lost. The rule is decision 0013 (standards/docs/decisions/0013-credit-window-scheduler.md on main); if it differs from this prompt, follow the decision.

1. Read the usage. Call list_events on your own session with kinds ["rate_limit_event"] and limit 100 (page back with before_id if the page has fewer than two events). From the newest event take rate_limit_info: isUsingOverage, status, unifiedWindows.five_hour {utilization u5, resetsAt}, unifiedWindows.seven_day {utilization u7, resetsAt}. h = hours until the five-hour resetsAt, d = days until the seven-day resetsAt. If there are no events or the fields are missing, use only the 5-thread floor and tell Jim once that the usage can't be read (don't repeat it every hour).
2. Count threads with list_threads: W = working (not blocked on Jim), O = open (not done, failed or stopped).
3. Apply the rule:
   a. STOP: if isUsingOverage is true or any status is not "allowed", start nothing and tell Jim (AGENTS.md overage rule).
   b. Burn rate r = rise in u5 per hour between the newest event and the newest one at least 10 minutes older with the same five_hour resetsAt. If there is no such pair, r = 0.02 * W. Per-thread cost t = r / W, or 0.02 if W is 0.
   c. Projection p = u5 + r * h.
   d. WEEKLY GUARD: e = 1 - d / 7. If u7 > e + 0.15 and d > 0.5, don't burst: go to g.
   e. BURST: if p < 0.85, start n = ceil((0.95 - p) / (t * max(h, 0.5))) new threads, at most 8 this run and never more than 30 open.
   f. EASE OFF: if p > 1.0 and h > 1, start no thread beyond the floor.
   g. FLOOR: otherwise keep at least 5 threads working, as AGENTS.md says.
4. Start threads the usual way: each runs the start-a-new-task skill, scoped to a project no other open thread is in; skip luggage, starcraft and profiles (on hold) and tooling files the mise migration owns. If 30 open threads is reached, tell Jim.
5. Report. In your turn, write one line with the numbers: u5, h, r, p, u7, e, W, O and how many threads you started. Post to the project chat only if you started threads, stopped for overage, or hit a limit; otherwise end the turn quietly.
```
