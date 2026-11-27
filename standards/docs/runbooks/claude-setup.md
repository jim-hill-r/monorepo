# Set up Claude

- **Kind:** one-time (done; keep for a rebuild). Connector sign-ins and the
  budget are re-checked every 30 days by settings checks.
- **Status:** done before 2026-11-26
- **Time:** about 30 minutes from scratch
- **Cost:** Claude Code is the one paid service, with a budget of $25 a
  month (AGENTS.md).

## What it's for

The agents that do all the repo's work: the Monorepo project in Claude,
its coordinator and threads, the GitHub access they act through, and the
scheduled tasks (PR triage, the weekly digest, the weekly deep review).

## Why a person

Signing up, paying, installing an app on a repo and signing in a
connector all need the account owner. Agents must not be able to widen
their own access.

## Before you start

- [Set up the GitHub repository](github-repo-setup.md) done.

## Steps

1. **Account and budget.** Sign up at claude.ai with a plan that includes
   Claude Code. Settings > Billing: keep extra usage off and spend within
   $25 a month.
2. **Claude GitHub App.** github.com/apps/claude > Install > Only select
   repositories > `jimhillr/monorepo`. Don't give it Administration,
   Environments or Secrets write (it doesn't ask for them; check
   `github-claude-app-permissions`). Connect GitHub to Claude at
   claude.ai/connect-github, signed in as `jimhillr-agt`, so agent PRs and
   approvals come from that account.
3. **Project.** claude.ai > Projects > New project "Monorepo", add the
   repository `jimhillr/monorepo`, and set the project instructions to
   "Follow AGENTS.md in the repo. It holds all project rules. Don't create
   Claude artifacts. Write plans, decisions and retros into the repo
   instead."
4. **Connectors.** Settings > Connectors: connect GitHub, and Cloudflare
   read-only.
5. **Scheduled tasks.** In the project, add (each a fresh session that
   follows AGENTS.md):
   - "PR triage", 7:50, 12:50 and 17:50 New York time, run as
     `jimhillr-agt`: the PR lifecycle in AGENTS.md and the repo health
     check. Paste the prompt from
     `standards/docs/scheduled-tasks/pr-triage.md`.
   - "Weekly human issues digest", Mondays 5:41 AM New York: run the
     `human-issues-digest` skill.
   - "Weekly deep project review", weekly: run the `deep-project-review`
     skill.
   - "Daily eval triage", daily at 7:30 New York (4:30 AM Pacific): paste
     the prompt from `standards/docs/scheduled-tasks/eval-triage.md`
     ([Schedule the daily eval triage](eval-triage-task.md)).

## Where secrets go

None in the repo. GitHub and Cloudflare sign-ins live in Claude's
connector settings.

## Verify

The settings checks `claude-github-app`, `claude-connectors`,
`claude-scheduled-tasks` and `claude-code-budget` answer "ok".

## Undo or rotate

To stop all agent work: pause the scheduled tasks and uninstall the
Claude GitHub App. Reconnect an expired connector in Settings >
Connectors.
