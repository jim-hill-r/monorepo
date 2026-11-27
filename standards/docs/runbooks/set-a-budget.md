# Set or raise a budget

- **Kind:** on demand (an agent asks for a number)
- **Status:** ongoing
- **Time:** about 2 minutes
- **Cost:** the budget itself, which caps a free-tier allowance, never
  money, except Claude Code ($25 a month)

## What it's for

Each technology's monthly budget is in the tech radar and
`standards/radar/limits.toml`; automation stops before it is reached
([free-tiers.md](../free-tiers.md)).

## Why a person

Only Jim sets or raises a budget (AGENTS.md), so an agent can't grant
itself more room.

## Before you start

- The agent's request: which budget key, the number, how close to the free
  limit it gets, and why.

## Steps

1. Reply with the number (for example "auth0_m2m_tokens_per_month: 60"),
   or approve the PR that sets it.
2. Never raise a vendor's own spending or cache limit, or add a payment
   method, to make room.

## Where secrets go

None.

## Verify

The PR that edits `standards/radar/limits.toml` is labelled `needs-human`
by the Cost summary job and merges after your approval.

## Undo or rotate

Reply with the lower number; an agent edits it back.
