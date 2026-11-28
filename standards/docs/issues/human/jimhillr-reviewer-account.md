---
added: 2026-11-27
unblocks: "jimhillr-reviewer approving low-risk agent PRs (decision 0014)"
runbook: standards/docs/runbooks/jimhillr-reviewer-account.md
---

## Requirement

Create the GitHub account `jimhillr-reviewer`, give it Write access to
jimhillr/monorepo, make its fine-grained token (Pull requests read and
write; Contents, Checks and Commit statuses read; this repo only), and put
the token in the `reviewer` environment (main only) as `REVIEWER_TOKEN`.
Then the All Auto Review workflow approves the green agent PRs its rules
allow, and the rest still wait for you.

## Why it can't be automated

A person has to sign up for the account and set up its two-factor sign-in,
only the account's owner can make its token, and adding a collaborator or
an environment needs Admin, which agents never get (decision 0010).

## Runbook

[Set up the jimhillr-reviewer account](../../runbooks/jimhillr-reviewer-account.md)
