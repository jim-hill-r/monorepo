# Review a PR that needs a person

- **Kind:** on demand (a PR labelled `needs-human`, or an agent PR waiting
  for an approving review)
- **Status:** ongoing. The weekly digest lists open `needs-human` PRs.
- **Time:** a few minutes to an hour, depending on the PR
- **Cost:** $0

## What it's for

Agents never approve their own PRs, so every agent PR merges only after a
reviewer (normally Jim) approves it; auto-merge then squash-merges it once
checks are green. `needs-human` marks a PR with a real decision for a
person: a vendor, money, a budget, an unsafe change or a manual step
([pull-requests.md](../pull-requests.md)).

## Why a person

It is a sign-off an agent may not give itself.

## Before you start

- Signed in to GitHub as `jimhillr`.

## Steps

1. Open the PR. Read the description first: it says what changed, what it
   costs, and any decision for you.
2. Read the diff. For `cost` or `needs-human`, check the Cost summary job's
   comment.
3. Approve (Review changes > Approve), or Request changes with what to fix,
   or answer the question in the PR or the Claude thread.
4. For a `credential-change` PR, the apply also needs [Approve a credential
   change run](approve-credential-run.md) after merge.

## Where secrets go

None.

## Verify

The PR merges by itself once approved and green (auto-merge), or PR
triage merges it on its next run (7:50, 12:50 and 17:50 New York).

## Undo or rotate

Revert the merged PR from its page (Revert button), and approve the revert.
