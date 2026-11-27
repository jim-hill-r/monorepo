# Approve a new vendor

- **Kind:** on demand
- **Status:** ongoing
- **Time:** about 10 minutes to read the proposal
- **Cost:** given in the proposal (usually $0 on a free tier)

## What it's for

Only GitHub, Cloudflare and Squarespace are approved, plus the exceptions
in AGENTS.md. Any other vendor is proposed in a decision record in
`standards/docs/decisions/` (or the PR for a small change) with its cost,
why no approved vendor fits, and what a switch would take, and the PR is
labelled `needs-human`.

## Why a person

A new vendor is a new account, a new bill risk and a new place for data;
AGENTS.md reserves it for Jim's sign-off.

## Before you start

- The decision record or PR.

## Steps

1. Read the options table: money cost, free-tier limits, Claude Code build
   effort for each.
2. Approve the PR, or reply with the option you choose.
3. If approved, the sign-up is a new human task with its own runbook here.

## Where secrets go

None at this step.

## Verify

The PR merges, the tech radar (`standards/radar/radar.json`) lists the
vendor, and AGENTS.md lists the exception.

## Undo or rotate

A later decision record supersedes it; the radar moves it to Hold.
