# Set up the GitHub repository

- **Kind:** one-time (done; keep for a rebuild or an audit)
- **Status:** done before 2026-11-26. The settings checks in
  `standards/settings.toml` (`github-*`) re-check each part every 30 or 90
  days.
- **Time:** about 30 minutes from scratch
- **Cost:** $0 on GitHub Free with a public repo

## What it's for

The repo every project, workflow, issue and agent lives in, with the
settings that keep agents inside their lane.

## Why a person

Creating accounts and repos, choosing roles and changing repo settings all
need the owner. The agent account must not be able to raise its own role.

## Before you start

- A GitHub account, `jimhillr`, with a passkey or security key.

## Steps

1. **Repo.** Create `jimhillr/monorepo`, public.
2. **Agent account.** Create a separate GitHub account `jimhillr-agt` for
   agents (its own email). In jimhillr/monorepo > Settings > Collaborators
   and teams, invite it with the **Write** role, never Maintain or Admin.
3. **Merge settings.** Settings > General > Pull Requests: allow squash
   merging and turn on "Allow auto-merge". ("Automatically delete head
   branches" is off for now; branch cleanup runs through the
   `all-delete-branches.yml` workflow.)
4. **Default ruleset.** Settings > Rules > Rulesets > New branch ruleset
   `Default`, active, target the default branch: require a pull request
   (1 approval), block force pushes, restrict deletions, and require the
   `CI result` status check ([Require CI on the main
   ruleset](github-ruleset-required-check.md)). Bypass list: empty.
5. **Actions budget.** github.com/settings/billing > Budgets and alerts:
   an Actions budget of $0 with "Stop usage when budget limit is reached".
   No payment method.
6. **Security.** [Turn on GitHub code security](github-code-security.md).
7. **Labels.** `needs-human`, `cost` and `credential-change` (agents can
   make labels, but create them if missing).

## Where secrets go

None here. Each secret's runbook names its place.

## Verify

The settings checks `github-main-ruleset`, `github-merge-settings`,
`github-actions-budget`, `github-dependabot` and `github-agent-role` all
answer "ok" ([Answer a settings check](settings-check.md)).

## Undo or rotate

To cut off agents at once: remove `jimhillr-agt` as a collaborator, or
uninstall the Claude GitHub App ([Set up Claude](claude-setup.md)).
