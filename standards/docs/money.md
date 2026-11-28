# Money: the financial ledger and runway

The repo keeps a public **financial ledger** of every money movement, and a
**runway** that says how long the money lasts
([decision 0012](decisions/0012-revenue-and-http-402.md), revenue phase 1 of
the [revenue plan](plans/2026-11-revenue-402.md)). It is kept double-entry
under the [accounting rules](cost.md#accounting-rules)
([decision 0016](decisions/0016-holding-and-handling-money.md)). The
`cost money` command (the [`cost`](../../cost/README.md) project) writes
it from files in the repo: no vendor, no credential, $0.

## Jim's rules (2026-11-28)

> "Until we get the actual financial accounts everything will be paid by me
> so we should have some accounting of that so I can get paid back
> eventually. Also, 1% of all revenue should be paid to me as salary with a
> ceiling of 1 million dollars per year. 1% for the planet as well. Our
> accounting should not make any assumptions about cost.. A budget is not
> actual spend. Our accounting should only count actual spend."

So:

1. **Only actual money.** The ledger books revenue that was received and
   spend that was charged, each with its receipt, invoice or bank record.
   Radar budgets and the expected `[[cost]]` entries in `docs/cost.toml`
   are plans; they are never booked.
2. **What Jim pays is owed back to him.** Spend Jim pays himself is a
   liability, `liability:owed-to-jim:expenses`, not a gift or equity, so
   the books always say how much the project owes him. Cash he puts into
   the project's account is a loan, `liability:owed-to-jim:loans`.
3. **Jim's salary is 1% of revenue**, at most $1,000,000 in a calendar
   year, accrued each month as `liability:owed-to-jim:salary`.
4. **1% for the Planet gets 1% of revenue**, with no ceiling, accrued each
   month as `liability:one-percent-for-the-planet`.

"Revenue" is gross income (`income:` lines): sponsors, x402, royalties and
gifts. Jim's loans and repayments are not revenue. The salary rounds down
to the cent and the Planet's share rounds up, so each gets at least its
rule. Both are split across projects by each project's revenue that month.

## Files

| File | Who writes it | What it holds |
| --- | --- | --- |
| `standards/money/runway.toml` | Jim, in a PR | `start` (the first month, `YYYY-MM`), `opening_cents` (money in the project's bank account at the start), `low_months` (default 3), optional `notes`. Unknown fields are errors. |
| `standards/money/actuals.toml` | An agent, from what Jim reports ([runbook](runbooks/record-actual-spend.md)) | `[[spend]]` and `[[payment]]` entries (below). |
| `<project>/docs/cost.toml` | Anyone, in a PR | `[[revenue]]` entries ([cost.md](cost.md#format)). |
| `<out>/statements/<yyyy-mm>.json` | `cost money` | One month of the ledger (below). |
| `<out>/runway.json` | `cost money` | Where the money stands after the last month (below). |

`[[spend]]` has `month` (the month it serves; a yearly charge is entered as
twelve monthly entries), `vendor` (a radar vendor), `cents`, `paid_by`
(`jim` or `bank`), `source` (the receipt or invoice) and an optional
`project`. Without `project` the spend is shared and split by merged PRs.

`[[payment]]` has `month`, `payee` (`jim-expenses`, `jim-loans`,
`jim-salary` or `planet`), `cents` and `source` (the bank record): money
the project's account paid against what it owes.

`<out>` is `target/money` unless `--out` says otherwise, so a local run
leaves nothing to commit. The monthly statement PR writes to
`standards/money`.

## Running it

```bash
# This month so far, into target/money/.
cargo run -p cost_cli -- money
# A closed month, into the committed folder.
cargo run -p cost_cli -- money --month 2026-11 --out standards/money
```

It needs the full git history (`actions/checkout` with `fetch-depth: 0`),
because the merged PRs split shared spend; on a shallow clone it stops
with an error rather than guess.

## Statements

A statement has the month's `opening` position, its `lines`, the `totals`
(`income_cents`, `expense_cents`) and the `closing` position, which is the
next month's opening. A position has `bank_cents`, `owed_to_jim_cents`,
`owed_to_planet_cents` and `net_cents` (the bank less what is owed). Each
line moves whole `cents` `from` one account `to` another, is booked to one
`project`, and names its `source`, so every statement balances and every
number links to where it came from.

| Money | From | To | Project |
| --- | --- | --- | --- |
| `[[revenue]]` (source `sponsors:…`, `x402:…`, `royalty:…`, `gift:…`) | `income:<stream>` (`income:other` without a stream) | `asset:bank` | The file's project |
| `[[revenue]]` with source `owner:…` (Jim's cash) | `liability:owed-to-jim:loans` | `asset:bank` | The file's project |
| `[[spend]]` Jim paid | `liability:owed-to-jim:expenses` | `expense:<vendor>` (`expense:claude-code`) | Its `project`, or split by merged PRs |
| `[[spend]]` the bank paid | `asset:bank` | `expense:<vendor>` | Its `project`, or split by merged PRs |
| Jim's salary | `liability:owed-to-jim:salary` | `expense:salary` | Split by the month's revenue |
| 1% for the Planet | `liability:one-percent-for-the-planet` | `expense:one-percent-for-the-planet` | Split by the month's revenue |
| `[[payment]]` | `asset:bank` | The payee's liability | `standards` |

The merged-PR split is the same rule as pane's Money page: a merged PR (a
first-parent commit whose subject ends `(#N)` or starts `Merge pull request
#`) counts `1/n` to each of the `n` top-level projects it touched, a PR
touching none counts to `standards`, and so does the whole amount in a
month with no merged PRs. Shares are whole cents by largest remainder, so
the projects add up to the total to the cent.

The check that the closing bank line matches the real balance Jim enters,
and that each statement's lines add up per project, is the next step
([issue](../../cost/issues/ledger-double-entry-balance-check.md)).

## Runway

`runway.json` has:

- `month`: the last month counted (the start month before there are any);
- `position`: the last statement's closing position;
- `burn_cents`: the average net cost a month (expenses less income) over
  the last three months, or fewer while the ledger is younger, rounded
  toward zero;
- `runway_months`: `net / burn` to one decimal, `null` when the burn is
  zero or less (the money isn't running down), `0` when the net position
  is below zero;
- `low_months` and `status`:

| Status | When |
| --- | --- |
| `out` | The net position is below zero: the project owes more than it has. |
| `funded` | The burn is zero or less, or the net position covers at least `low_months` of burn. |
| `low` | Anything else. |

While Jim pays for everything and there is no revenue, the status is `out`
as soon as any spend is recorded: it is the honest answer (the project owes
Jim more than it has). Phase 2 of the revenue plan, which makes `out`
pause new agent work, has to decide with Jim whether money owed to him
counts for that pause.

## What comes next

Later phases of the revenue plan read these files: a scheduled run writes
the statements and uploads the JSON, pane serves `/api/runway` and the
public ledger page, and the apps show a banner and splash.
