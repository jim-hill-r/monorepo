---
kind: todo
priority: priority
tags: [agent-generated]
added: 2026-11-27
---

When the financial ledger from decision 0012 phase 1 is built (monthly statements in standards/money/statements/), keep it double-entry as decision 0016 and the Accounting rules in standards/docs/cost.md ask: each statement line names the account it moves from and to, and a new cost check rule fails when an entry's postings don't sum to zero, when per-project income or expense doesn't sum to the statement's totals, or when the closing bank line differs from the balance Jim entered. Add a cost requirement and evals for it, and a cost money journal export in hledger format. See ../../standards/docs/decisions/0016-holding-and-handling-money.md.
