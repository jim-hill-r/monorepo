# Record what you actually paid each month

- **Kind:** recurring (every month, early in the month for the one before)
- **Status:** open (issues/human/record-actual-spend.md)
- **Time:** about 5 minutes
- **Cost:** $0 (reading receipts; no service or free tier is used)

## What it's for

The financial ledger (`cost money`, [money.md](../money.md)) books only
money actually spent, never a budget or an estimate (Jim, 2026-11-28).
Until the project has its own bank account
([decision 0016](../decisions/0016-holding-and-handling-money.md)), Jim
pays for everything, and each payment is booked as owed back to him. Without
these numbers the ledger shows nothing spent and nothing owed.

## Why a person

The charges are on Jim's own card and accounts. Agents never hold a
credential with billing access, Claude shows agents only whether a session
is in overage, not dollars, and Squarespace has no billing API.

## Before you start

- Signed in to claude.ai (Settings, Billing) and any other vendor that
  charged you for the project, such as Squarespace for a domain renewal.

## Steps

1. Open each vendor's billing or invoices page: claude.ai, Settings,
   Billing, for Claude Code; Squarespace, Domains, Billing for a domain.
2. For each charge for the month, note the amount, the date, and the
   receipt or invoice number. A yearly charge is fine as one amount; say it
   is yearly and the agent spreads it over twelve months.
3. Say which project a charge was for if it was for only one (a domain);
   Claude Code is shared and is split by merged PRs.
4. Post them in the Claude project, for example "November 2026: Claude
   Code $25.00, receipt 1234-5678". An agent adds a `[[spend]]` entry with
   `paid_by = "jim"` for each to `standards/money/actuals.toml` in a PR.
5. If the project's bank account paid you back, give the amount and the
   transfer date too; the agent adds a `[[payment]]`.

## Where secrets go

None. Amounts and receipt numbers are not secrets, and the ledger is
public. Leave out card numbers, account numbers and your address.

## Verify

The agent's PR runs `cost money` for the month: the statement shows each
charge as a line from `liability:owed-to-jim:expenses`, and `runway.json`
shows the running total in `owed_to_jim_cents`.

## Undo or rotate

A wrong entry is fixed by a PR that edits `standards/money/actuals.toml`;
git history keeps the old value as the audit trail.
