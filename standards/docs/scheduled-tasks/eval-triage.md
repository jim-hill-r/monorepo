# Daily eval triage scheduled task

The prompt of the "Daily eval triage" scheduled task in the Claude project
(daily at 7:30 America/New_York, which is 4:30 AM Pacific: after Nightly CI
at 2:00, Nightly CD at 2:30 and the planned 4:00 observe run; a fresh session
each run). The work is the `eval-triage` skill
(`.agents/skills/eval-triage/SKILL.md`); this file is the exact prompt, kept
in the repo so a change to it is reviewed like code. When this file changes,
an agent updates the scheduled task's prompt to match once the PR merges, and
says so in its thread. Jim adds the task by hand; see
[Set up Claude](../runbooks/claude-setup.md), step 5.

History:

- 2026-11-27: created (eval engine phase 2; replaces the planned nightly FIX
  task of the deploy feedback plan).

## Prompt

```text
Daily eval triage for the GitHub repo jimhillr/monorepo. Attach the repo (add_repo owner=jimhillr repo=monorepo access=push), read AGENTS.md on main, then run the `eval-triage` skill in .agents/skills/eval-triage/SKILL.md exactly as written. If Claude Code reports overage usage, stop before step 1 and say so in one line (AGENTS.md, "Pause on Claude overage"). If the skill finds nothing to add, open no PR and end with one line saying so. Never roll a deploy back, rerun a workflow, approve a PR or change a product or an eval yourself.
```
