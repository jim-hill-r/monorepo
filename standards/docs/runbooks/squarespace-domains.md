# List the Squarespace domains

- **Kind:** one-time, then re-checked every 90 days by the
  `squarespace-billing` settings check (phase 4 of the cost tracking plan)
- **Status:** open (`cost/docs/issues/human/list-squarespace-domains.md`)
- **Time:** about 10 minutes
- **Cost:** $0 to do. It records money already being paid: each domain's
  yearly renewal.

## What it's for

Squarespace domains are the one real money the repo pays outside Claude
Code, and none of it is recorded yet. Once listed, each domain becomes a
yearly `[[cost]]` in a `docs/cost.toml` ([cost.md](../cost.md)), so the
monthly total is complete, and the digest warns before each renewal
([cost tracking plan](../plans/2026-11-cost-tracking.md), phase 4).

## Why a person

Squarespace has no public API for domains or billing, and agents never
hold a credential with billing access ([free-tiers.md](../free-tiers.md)).

## Before you start

- Signed in to the Squarespace account that holds the domains.

## Steps

1. Open account.squarespace.com > Domains.
2. For each domain, note:
   - the domain name;
   - the yearly renewal price (open the domain > Billing, or the renewal
     line on the domain's page);
   - the next renewal date;
   - whether auto-renew is on;
   - what it is for: which project or site uses it, a redirect, email, or
     nothing.
3. Open account.squarespace.com > Billing and note any other paid item
   (a website plan, Google Workspace email, a domain add-on such as
   privacy) with its price and renewal date.
4. Choose a monthly Squarespace budget for the tech radar: the yearly total
   divided by 12, rounded up, is enough.
5. Reply in the Claude project with the list and the budget. Don't change
   any domain or setting while doing this.

## Where secrets go

None. Don't send account numbers, card details or Squarespace login codes;
the domain names, prices and dates are enough.

## Verify

An agent opens a PR that adds each domain as a `[[cost]]` with a decision
record and sets the Squarespace budget in `standards/radar/radar.json`.
`cost check` passes on it (COST-004 and COST-006) once you accept the
decision record and approve the PR. Then tell any thread "done" so it
deletes the issue.

## Undo or rotate

Nothing to undo. To stop paying for a domain, turn off auto-renew on its
page before the renewal date and tell an agent, which removes its
`[[cost]]` once it lapses.
