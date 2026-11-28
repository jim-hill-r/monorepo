# Set up the jimhillr-reviewer account

- **Kind:** one-time, then a yearly token rotation
- **Time:** about 15 minutes
- **Cost:** $0 (a free GitHub account; environments and Actions are free on
  public repos)
- **Decision:** [0014](../decisions/0014-faster-pr-approvals.md), Jim's
  choice on 2026-11-27

## What it's for

`jimhillr-reviewer` is a third GitHub account whose only job is to approve
agent PRs that pass its written rules ("The reviewer account" in
[pull-requests.md](../pull-requests.md)). The `All Auto Review` workflow
runs as it every 15 minutes. Its approvals satisfy the Default ruleset's
one approval, so nothing in the ruleset changes, and every approval says
which rules it passed. Jim still approves everything else, and reads every
merged PR afterwards on pane's Acknowledge page.

## Why a person

A GitHub account needs a person to sign up, accept the terms and set up
two-factor sign-in, and only an account's owner can make its personal
access token. Inviting a collaborator and making an environment need
Admin, which agents never get (decision 0010). GitHub's terms allow a
machine account run by a person who is responsible for it; check the
current terms when you sign up.

## Before you start

- An email address for the account that isn't used by `jimhillr` or
  `jimhillr-agt`.
- Signed in to GitHub as `jimhillr` in a second browser profile, for the
  repo settings.

## Steps

1. **Account.** Sign up at github.com as `jimhillr-reviewer`, Free plan.
   Turn on two-factor authentication (a passkey or an authenticator app).
2. **Access.** As `jimhillr`: jimhillr/monorepo > Settings > Collaborators
   and teams > Add people > `jimhillr-reviewer`, role **Write** (an
   approval only counts from an account that can write; never Maintain or
   Admin). Accept the invite as `jimhillr-reviewer`.
3. **Token.** As `jimhillr-reviewer`: Settings > Developer settings >
   Personal access tokens > Fine-grained tokens > Generate new token.
   - Name `jimhillr-reviewer-approve`, expiry 1 year.
   - Resource owner `jimhillr`, repository access: only
     `jimhillr/monorepo`.
   - Repository permissions: **Pull requests: Read and write**;
     **Contents**, **Checks** and **Commit statuses: Read-only**. Nothing
     else: without Contents write it can approve but never push or merge.
4. **Environment.** As `jimhillr`: jimhillr/monorepo > Settings >
   Environments > New environment `reviewer` (or open it if a first
   workflow run made it). Deployment branches and tags: **Selected
   branches**, add `main`. No required reviewers (that would stop every
   run for you). Add environment secret `REVIEWER_TOKEN` with the token
   from step 3. Close the token page without saving the value anywhere
   else.
5. Leave the Default ruleset as it is: one approval, stale approvals
   dismissed, bypass list empty.
6. **Email.** As `jimhillr-reviewer`, turn off GitHub email
   notifications ([github-email-notifications-off.md](github-email-notifications-off.md),
   steps 1 to 3).

## Where secrets go

`REVIEWER_TOKEN` in the `reviewer` environment of jimhillr/monorepo,
nowhere else: never in git, OpenTofu state, a chat, or `jimhillr-agt`'s
reach. Only the `All Auto Review` workflow on `main` reads it.

## Verify

1. Actions > All Auto Review > Run workflow, with "Decide but don't
   approve" ticked. The run summary lists each open agent PR with
   "approve" or "skip" and why.
2. Run it again without the box. An approvable PR gets a review from
   `jimhillr-reviewer` whose text starts "jimhillr-reviewer rules v1".
3. Tell an agent: it deletes
   [the human task](../issues/human/jimhillr-reviewer-account.md).
   The settings checks `github-reviewer-account` and
   `rotate-reviewer-token` cover it from then on.

## Undo or rotate

- **Stop it at once:** delete the `REVIEWER_TOKEN` secret, or remove
  `jimhillr-reviewer` as a collaborator. The workflow then approves
  nothing and every PR waits for you again.
- **Rotate (yearly, or on a leak):** as `jimhillr-reviewer`, regenerate
  the token with the same settings and replace `REVIEWER_TOKEN`
  ([rotate-a-secret.md](rotate-a-secret.md)).
