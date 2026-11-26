# 0002. Manual tasks for a human live in HUMAN_ISSUES.md

- Status: Accepted
- Date: 2026-11-26
- Decided by: Jim

## Context

Agents keep hitting tasks only a person can do: Auth0 and Cloudflare
dashboard settings, creating secrets, billing checks, sign-offs. These were
recorded in three places and nothing collected them:

- `TODO (agent-ignore): ... Needs Jim.` lines in `ISSUES.md` files, mixed in
  with agent work.
- The `needs-jim` label that Hourly PR triage puts on PRs.
- Questions in Claude chat threads, which are lost once the thread ends.

Options considered: a pinned GitHub issue kept up to date by a workflow, with
a weekly digest comment (free, and GitHub notifies by email), or a file in the
repo with a Claude scheduled task that reports on it. Jim chose against GitHub
issues: work is tracked in the repo, not in GitHub issues.

## Decision

- Every manual task is a `TODO (human): ...` line in the root
  `HUMAN_ISSUES.md`. Agents never work them and never put them in an
  `ISSUES.md` (the audit checks this, HUM-001).
- PRs that need a person get the label `needs-human`, which replaces
  `needs-jim` (Jim, 2026-11-26: "make it more generic"). Hourly PR triage
  renames the old label on its next run, so the open PRs keep it.
- A weekly Claude scheduled task, "Weekly human issues digest", runs the
  `human-issues-digest` skill and posts the open lines and `needs-human` PRs
  in the Claude project.

## Consequences

- One place to look, and one weekly message, at no cost and with no new
  vendor.
- The digest only reaches Jim through Claude; there is no email or GitHub
  notification. If that turns out to be too easy to miss, the skill's message
  can also go to the Claude app's push notifications (the scheduled task has
  them on).
- Closing a task needs a PR that removes its line. Agents do that when Jim
  says a task is done.
