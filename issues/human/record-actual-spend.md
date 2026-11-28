---
added: 2026-11-28
unblocks: An accurate financial ledger and runway, and the record of what the project owes Jim
runbook: standards/docs/runbooks/record-actual-spend.md
---

## Requirement

Each month, send the actual amount you paid for every project service (today Claude Code, and any domain renewal) with its receipt or invoice number, and an agent adds a [[spend]] entry with paid_by = "jim" to standards/money/actuals.toml. Start with November 2026. The financial ledger (cost money, standards/docs/money.md) books only actual spend, so without these it shows nothing spent and nothing owed back to you.

## Why it can't be automated

The charges are on Jim's own card and accounts, and agents never hold a credential with billing access. Claude's spend in dollars isn't readable through any API agents have (only overage flags), and Squarespace has no billing API.

## Runbook

[Record what you actually paid each month](../../standards/docs/runbooks/record-actual-spend.md)
