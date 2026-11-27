# Require CI on the main ruleset

- **Kind:** one-time
- **Status:** open (`issues/human/github-settings-rules-rulesets-default-ruleset.md`)
- **Time:** about 3 minutes
- **Cost:** $0

## What it's for

Makes auto-merge wait for green CI after any push. Until it's on, agents
turn on auto-merge only while checks are green (the Auto-merge section of
[pull-requests.md](../pull-requests.md)).

## Why a person

Rulesets need Admin, and agents must never be able to change the rules
that hold them back (decision 0010).

## Before you start

- Signed in to GitHub as `jimhillr`.
- The `CI result` check has run at least once on a PR, so GitHub can
  offer it (it runs on every PR).

## Steps

1. jimhillr/monorepo > Settings > Rules > Rulesets > Default.
2. Tick "Require status checks to pass". Add checks > type `CI result` >
   pick the one from GitHub Actions.
3. Leave the bypass list as it is (empty of agents), and Save changes.

## Where secrets go

None.

## Verify

Open any PR with a pending CI run: the merge box says "Required" next to
`CI result`. The `github-main-ruleset` settings check covers the ruleset
every 90 days.

## Undo or rotate

Untick "Require status checks to pass" in the same ruleset.
