# Schedule the daily eval triage

- **Kind:** one-time. The `claude-scheduled-tasks` settings check then
  re-checks it every 30 days with the other scheduled tasks.
- **Status:** open (issues/human/schedule-daily-eval-triage.md)
- **Time:** about 5 minutes
- **Cost:** $0 extra. It runs inside the Claude Code subscription (budget
  $25 a month, extra usage off), so it uses some of the plan's usage and no
  money. A quiet day (nothing not met, no eval in error) reads two files and
  stops in a few minutes; a day with findings opens one small PR of issue
  files. If usage gets tight, run it weekly instead (same prompt, Mondays).

## What it's for

The eval engine records whether each product requirement holds in
production, but nothing acts on a red verdict yet. The daily triage reads
the verdicts each morning and files a `FIX` for each `must` requirement not
met, a `TODO` for each broken eval, and says how to roll back a deploy that
failed its evals, in one draft PR. Without it, a broken production
requirement shows only in pane and the job logs until someone looks.

## Why a person

Claude scheduled tasks are made in the claude.ai project settings, which
have no API an agent can call, and agents must not schedule their own
recurring work (it spends the Claude budget, which only Jim controls).

## Before you start

- Signed in to claude.ai as the owner of the Monorepo project.
- [Set up Claude](claude-setup.md) done (the GitHub App and connector).

## Steps

1. claude.ai > Projects > Monorepo > Scheduled tasks > New task.
2. Name it "Daily eval triage". Schedule: every day at 7:30 AM,
   America/New_York (4:30 AM Pacific).
3. Paste the prompt from the `text` block in
   [standards/docs/scheduled-tasks/eval-triage.md](../scheduled-tasks/eval-triage.md)
   on `main`.
4. Save, then use "Run now" once.

## Where secrets go

None. The task uses the project's existing GitHub access.

## Verify

The first run ends with either "nothing to triage" or a link to a draft PR
titled "Eval triage, <date>". Tell any thread "done" and it deletes
`issues/human/schedule-daily-eval-triage.md` and sets this runbook to done.

## Undo or rotate

Pause or delete the task in the same Scheduled tasks list. Nothing else
depends on it.
