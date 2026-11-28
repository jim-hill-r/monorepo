# 0016. Holding and handling money: a separate free account, accounting books in the repo, a merchant of record for card sales

- Status: Accepted
- Decided by: Jim (2026-11-27: question 1 on a decision card, "Separate
  free account with EIN"; questions 2 to 5 in the project thread, "All
  recommended"). The accounting rules are Jim's own.
- Date: 2026-11-27
- Asked by: Jim (project thread, 2026-11-27: "Have we done a tech selection
  on how we are going to handle our finances? How do we store money and
  report on it etc...")
- Builds on: [0012](https://github.com/jimhillr/monorepo/pull/764) (revenue: the financial
  ledger, GitHub Sponsors, x402 staged; still in PR #764),
  [0004](0004-database-engine.md) (files before databases),
  [0011](0011-mint-secrets-by-workflow.md) (secrets)
- Human task: [open a project bank account and get an EIN](../../../cost/docs/issues/human/open-project-bank-account-and-ein.md)

Not legal or tax advice. The tax lines are what a tax professional should
confirm before real money moves.

## Context

Decision 0012 settled how money is *recorded* and how it *comes in*: a
public **financial ledger** as repo files (`[[revenue]]` and `[[cost]]`
entries in each `docs/cost.toml`, monthly statements in
`standards/money/statements/` opened as PRs, an open-books page on pane),
GitHub Sponsors first, and x402 (HTTP 402, USDC) through Coinbase CDP only
once an API has outside demand. It left four things open:

1. **Where money is held.** Sponsors pays out through Stripe Connect into a
   bank account, and 0012 assumed Jim's own. Nothing keeps project money
   apart from his personal money.
2. **The processor** for anyone who wants to pay by card: a royalty
   invoice, or an API buyer without a USDC wallet.
3. **Reconciliation**: how a statement is proved against real balances,
   and which of those balances tooling can read.
4. **Tax and entity basics**: what Jim files as, and when an LLC is worth it.

Two words used below. The **financial ledger** is the repo's record of
every money movement (0012). The **accounting books** are the same data
kept double-entry (every amount leaves one account and enters another), so
a balance can be checked and a tax professional can read it.

Today's numbers: income $0; the only spend is Claude Code at $25 a month,
paid by Jim. So every choice below is judged first on money cost at that
scale, then vendor count, automation, open source and the secrets rules.

## Principles each option is judged on

- **Cost first** ($0 now; fees only as a share of money actually received).
- **Few vendors**: GitHub, Cloudflare and Squarespace are approved; a bank,
  fiscal host or processor is a new vendor and needs Jim's sign-off.
- **Automatable**: a workflow can read the balance with a read-only token,
  or from public data, without a person.
- **Open and in git**: the books are files we own in an open format.
- **Secrets**: no token in git or OpenTofu state; a read token goes into a
  GitHub secret through Infra Mint or the second approval (0010, 0011).
  No credential that can *move* money is ever given to automation.
- **Public repo**: the ledger is public (0012), so nothing personal (Jim's
  own transactions, account numbers, his SSN) may ever reach it.

## 1. Where the money is held

| Option | Money cost | Who holds it | Read by tooling | Fits? |
| --- | --- | --- | --- | --- |
| **A. A separate free checking account in Jim's name as a sole proprietor, with a free EIN** (recommended) | $0 (a free account at Jim's own bank or credit union, or Relay Starter: $0 a month, sole proprietors allowed, $5 per incoming wire) | Jim, but apart from his own money | Month-end balance and a CSV export; most free accounts have no read API | **Yes**. Every transaction in it is project money, so the whole account can be reconciled in public without exposing Jim's own spending. The EIN goes on the Sponsors W-9 instead of his SSN. |
| B. Jim's personal account (what 0012 assumed) | $0 | Jim, mixed with personal money | Not without exposing personal transactions | Works for $0 of income, but the books can't be checked against it in public, and the IRS prefers business money kept apart. |
| C. Mercury business account | $0 basic plan; a free API with scoped tokens | A registered business | **Yes**: a read-only API token in a GitHub secret | Only after an LLC: Mercury asks for formation documents and an EIN, and reviews disagree on whether sole proprietors qualify. The best fit for automation once there is an LLC. |
| D. Open Source Collective as fiscal host | 10% host fee on income, plus Stripe or PayPal fees | The collective, not Jim | **Yes**: a public ledger and a public GraphQL API, no token | Real third-party custody and open books for free, but it takes 10% of every dollar and spending becomes expense claims. Worth it only if Jim wants the money out of his hands. |
| E. Leave it in a Stripe balance or a crypto wallet | $0 | Stripe or the wallet | Stripe API (a key) or a public chain (no key) | **No** as the main account: Stripe pays out on a schedule and a balance isn't a bank account; the USDC wallet from 0012 phase 5 stays a receiving address that Jim sweeps out. |

Bank feed services (Plaid, SimpleFIN Bridge and the like) would let a
workflow read a bank with no API, but they are paid and hold a credential
that can see the whole account. Not worth it while the account sees a few
transfers a month.

**Recommendation: A now, C when an LLC exists.** A free EIN and a free
separate account cost nothing and take about an hour, and they are the
one step that is hard to fix later: once Sponsors payouts land in Jim's
personal account, the history is mixed. Set it up before GitHub Sponsors
(0012 phase 2), so the first payout lands there.

## 2. The accounting books

| Option | Money cost | Format | Fits? |
| --- | --- | --- | --- |
| **Repo files (0012's ledger) kept double-entry, with an export to a plain-text accounting journal** (recommended) | $0 | TOML and JSON we own; export in hledger journal format, which hledger, Ledger and (through a converter) Beancount read | **Yes**: in git, public, reviewed as PRs, checked by our own Rust `cost money` command, no vendor. |
| hledger or Beancount files as the source of truth | $0, open source (GPL) | Plain-text journal | Good tools, but a second format beside `cost.toml`, and a Haskell or Python tool in CI where we keep scripts Rust-only. Use them to *read* our export, not as the source. |
| QuickBooks, Xero, Wave | $0 (Wave's free tier) to about $20 to $40 a month | A vendor's database | **No**: a hosted database and a new vendor for a few lines a month; the books would leave the repo. |

The statement format from 0012 gains one rule: each line names the
account it moves from and to (`bank:checking`, `income:sponsors`,
`expense:claude-code`, `equity:owner-contribution`), so every statement
balances to zero and its closing bank line can be checked against the real
balance. A `cost money journal` export writes the hledger journal for a
tax professional at year end.

### Accounting rules: GAAP, double entry and full attribution (Jim, 2026-11-27)

Jim: "All of our cost software should follow GAP accounting so that
everything is double counted on the ledger and also we should have
attribution in such a way that if you sum up all service costs/revenue it
equals total revenue/costs." This is a rule, not a question; it is written
out under "Accounting rules" in [cost.md](../cost.md) and applies to the
`cost` project, pane's money code and the ledger:

- **Double entry**: every entry's postings sum to zero.
- **Accrual and matching** (GAAP): costs are booked in the months they
  serve and revenue when earned; the bank reconciliation is the cash view.
  A sole proprietor usually files taxes on a cash basis, so the year-end
  export gives both.
- **Full attribution**: every cent belongs to exactly one project and the
  projects sum to the totals. Shared costs (today only Claude Code) are
  split by share of merged PRs; repo-wide PRs, and a month with no PRs,
  are charged to `standards`, so nothing is left over.
- **Checked deterministically**: pane's `attribution_gaps` (requirement
  PANE-013) checks the per-project money against the month's totals now;
  the ledger's balance check lands with 0012 phase 1
  ([issue](../../../cost/docs/issues/ledger-double-entry-balance-check.md)).

## 3. The processor for card payments (402 and royalties)

0012 already picks x402 with the Coinbase CDP facilitator for API sales
(phase 5, gated) and GitHub Sponsors for gifts. What's missing is a way for
a person or company that wants to pay *by card* or *by invoice*: royalty
payers (organizations usually want an invoice) and API buyers without a
USDC wallet.

| Option | Fee on a $10 payment | Sales tax | Vendor | Fits? |
| --- | --- | --- | --- | --- |
| **GitHub Sponsors, one-time or organization tier** (recommended first) | $0 from personal accounts; organizations pay up to 6% on top | Not a sale, so none | Approved (GitHub) | **Yes** for royalties: organizations can sponsor by invoice, GitHub keeps the records, and royalty's report can point to a tier. |
| **Polar as merchant of record** (recommended when a card sale is needed) | 5% + $0.50 = $1.00 on its free Starter plan, plus 1.5% for non-US cards | **Polar collects and files it** | New vendor, open source (Apache-2.0) | **Yes, later**: it carries the sales tax that direct sales would leave on Jim (0012's main legal worry), has no monthly fee, and pays out to the account in section 1. |
| Stripe directly (Payment Links, Invoicing) | 2.9% + $0.30 = $0.59, more for invoices and Stripe Tax | **On Jim**: register and file in each state that taxes digital services | New vendor | Cheaper per payment, but sales tax filing would cost more in time than the fee saves at our size. |
| Lemon Squeezy, Paddle | About 5% + $0.50 | Merchant of record | New vendor | Same role as Polar, closed source; no reason to prefer them. |

**Recommendation:** no processor now. Royalties go through Sponsors tiers.
The first time someone wants to pay by card, propose Polar in its own
record with its own human task; it needs no card on file, which keeps the
no-payment-method rule.

## 4. Reconciliation: proving the books against real balances

| Account | Source of the real number | Automatable? |
| --- | --- | --- |
| GitHub Sponsors income | Sponsors payouts page; GraphQL API with a token of Jim's account | Jim enters each payout in a pane form (0012 decision 5); later a fine-grained read token through `infra-credentials` |
| Bank account (A) | Month-end balance from the bank site or statement | Jim enters it in the same pane form; the monthly statement PR fails its check when the books and the balance differ |
| Bank account (C, Mercury) | Read-only API token | Yes: a workflow reads the balance, token minted into a GitHub secret, never in state |
| USDC wallet (0012 phase 5) | The public chain | Yes, with no credential: the address is public |
| Open Source Collective (D) | Public GraphQL API | Yes, with no credential |
| Claude Code spend | The billing check and Jim's settings check (PR #760) | Partly, as today |

So with option A, reconciliation is two numbers a month from Jim (the
payout and the balance), about 2 minutes, and the statement PR checks they
match. Nothing that can move money is ever given to a workflow.

## 5. Tax and entity basics

- **Entity: sole proprietor now** ($0). It needs no filing. A free EIN
  (IRS online, minutes) keeps Jim's SSN off W-9s and opens a business
  account. **An LLC later** (state filing fee plus a yearly fee, which vary
  by state) once there is a paid API with outside users, real revenue, or a
  need for Mercury's API. The LLC is for liability, not tax: a
  single-member LLC is taxed the same as a sole proprietor.
- **Business or hobby** decides whether Claude Code is deductible
  (Schedule C) and whether self-employment tax applies to net profit over
  $400. A tax professional should answer this before the first real payout.
- **Forms**: GitHub sends a 1099-NEC above the yearly threshold (raised
  from $600 to $2,000 for payments from 2026, per the One Big Beautiful
  Bill Act; confirm with the professional). All income is reportable with
  or without a form.
- **Estimated taxes** are due quarterly once the year's tax owed passes
  $1,000; the statements show when that is near.
- **Sales tax** only arises with direct card or x402 sales; a merchant of
  record (Polar) takes it on.
- **Records**: the statements and the yearly journal export are the
  records; keep the bank's own statements outside the repo, since they
  carry account numbers.

## Questions for Jim

Each answer's money cost, free-tier position and Claude Code effort
(which counts against the $25 a month budget):

1. **Where money is held.** (a) **A separate free account as a sole
   proprietor, with an EIN, before Sponsors** (recommended; $0, about an
   hour of your time, no agent effort). (b) Your personal account, as 0012
   assumed ($0, no time, but the books can't be checked in public).
   (c) Open Source Collective (10% of income, a new vendor, about one
   session to wire its API). (d) An LLC and Mercury now (state fees, a new
   vendor, about one session).
2. **Which bank for (a).** (a) **Your own bank or credit union, as a
   second free account** (recommended; no new kind of vendor, $0).
   (b) Relay Starter ($0 a month, sole proprietors allowed, new vendor).
3. **Accounting books.** (a) **Repo files kept double-entry, with an
   hledger export** (recommended; $0, no vendor, about one session in
   0012's phase 1). (b) hledger or Beancount files as the source ($0, a
   second format and a non-Rust tool in CI).
4. **Card payments.** (a) **None now; Sponsors tiers for royalties; Polar
   proposed when a card sale is first needed** (recommended; $0 now,
   5% + $0.50 per sale later). (b) Stripe directly when needed (2.9% +
   $0.30, sales tax filing on you).
5. **Tax.** (a) **Talk to a tax professional before the first real
   payout** (recommended; a one-time fee you choose, outside our budgets).
   (b) Wait until phase 5 as 0012 says.

## Consequences

- 0012's answer 8 ("no business set-up until revenue is real") narrows:
  the EIN and separate account come before Sponsors; the LLC still waits.
- The human task above is done before 0012's phase 2 Sponsors task, and
  that task's runbook points the payout at the new account.
- 0012 phase 1 adds the double-entry accounts, the balance check and the
  `cost money journal` export, and pane's payout form gains a month-end
  balance field.
- The bank (if new), Polar and any fiscal host each get a radar entry and
  a billing check line in the PR that first uses them, not before.
- A settings check for the account (payout destination still the project
  account, no overdraft or card added) joins `standards/settings.toml`
  when the account exists.

## Sources

Checked 2026-11-27:
[Mercury FAQ](https://vercel.mercury.com/faq),
[Mercury API docs index](https://mercury.com/llms.txt),
[Mercury review (sole proprietors)](https://howtostartanllc.com/reviews/mercury-review),
[Relay Starter plan](https://relayfi.com/hc/en-us/articles/35672368525972-Overview-of-the-Starter-Plan/),
[Relay review](https://www.nav.com/business-checking/relay-financial-business-checking-account-review),
[Open Collective pricing](https://documentation.opencollective.com/why-open-collective/pricing),
[Open Source Collective fees](https://opencollective.com/opensource/updates/fees-arent-changing-but-how-we-show-them-is),
[Polar fees](https://polar.sh/docs/fees). GitHub Sponsors, x402 and the
tax background are sourced in the
[revenue plan](https://github.com/jimhillr/monorepo/pull/764) (PR #764).
