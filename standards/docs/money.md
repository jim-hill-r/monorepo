# Money: the financial ledger and runway

The repo keeps a public **financial ledger** of every money movement, and a
**runway** that says how long the money lasts
([decision 0012](decisions/0012-revenue-and-http-402.md), revenue phase 1 of
the [revenue plan](plans/2026-11-revenue-402.md)). It is kept double-entry
under the [accounting rules](cost.md#accounting-rules)
([decision 0016](decisions/0016-holding-and-handling-money.md)). The
`cost money` command (the [`cost`](../../cost/README.md) project) writes
it from files in the repo: no vendor, no credential, $0.

## Files

| File | Who writes it | What it holds |
| --- | --- | --- |
| `standards/money/runway.toml` | Jim, in a PR | `start` (the first month, `YYYY-MM`), `opening_cents` (money in the bank at the start), `low_months` (default 3), optional `notes`. Unknown fields are errors. |
| `<project>/docs/cost.toml` | Anyone, in a PR | `[[revenue]]` and `[[cost]]` entries ([cost.md](cost.md#format)). |
| `standards/radar/radar.json` | Tech radar PRs | Each vendor's monthly `budget`: the shared costs (today Claude Code's $25). |
| `<out>/statements/<yyyy-mm>.json` | `cost money` | One month of the ledger (below). |
| `<out>/runway.json` | `cost money` | Where the money stands after the last month (below). |

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
because the merged PRs split the shared costs; on a shallow clone it stops
with an error rather than guess.

## Statements

A statement has the month's `opening_cents`, its `lines`, the `totals`
(`income_cents`, `owner_cents`, `expense_cents`) and `closing_cents`, which
is the next month's opening. Each line moves whole `cents` `from` one
account `to` another, is booked to one `project`, and names its `source`,
so every statement balances and every number links to where it came from.

| Money | From | To | Project |
| --- | --- | --- | --- |
| `[[revenue]]` (source `sponsors:…`, `x402:…`, `royalty:…`, `gift:…`) | `income:<stream>` (`income:other` without a stream) | `asset:bank` | The file's project |
| `[[revenue]]` with source `owner:…` | `equity:owner-contribution` | `asset:bank` | The file's project |
| `[[cost]]`, booked in the months it serves (yearly over twelve months, once in its month) | `asset:bank` | `expense:<vendor>` (`expense:squarespace`) | The file's project |
| A radar vendor's monthly budget (`expense:claude-code`) | `asset:bank` | `expense:<vendor>` | Split across projects by the month's merged PRs |

Owner contributions are Jim's own money put in, as decision 0012's answer 2
says; they are never counted as income.

The shared-cost split is the same rule as pane's Money page: a merged PR
(a first-parent commit whose subject ends `(#N)` or starts `Merge pull
request #`) counts `1/n` to each of the `n` top-level projects it touched,
a PR touching none counts to `standards`, and so does the whole cost in a
month with no merged PRs. Shares are whole cents by largest remainder, so
the projects add up to the budget to the cent. The radar keeps no history,
so every month uses today's budgets.

Statements are on the accrual basis (rule 2): a yearly cost shows a
twelfth each month whatever day it was paid. The check that the closing
bank line matches the real balance Jim enters, and that each statement's
lines add up, is the next step
([issue](../../cost/issues/ledger-double-entry-balance-check.md)).

## Runway

`runway.json` has:

- `month`: the last month counted (the start month before there are any);
- `balance_cents`: the last statement's closing balance;
- `burn_cents`: the average net cost a month (expenses less income and
  owner contributions) over the last three months, or fewer while the
  ledger is younger, rounded toward zero;
- `runway_months`: `balance / burn` to one decimal, `null` when the burn is
  zero or less (the money isn't running down), `0` when the balance is
  below zero;
- `low_months` and `status`:

| Status | When |
| --- | --- |
| `out` | The balance is below zero. |
| `funded` | The burn is zero or less, or the balance covers at least `low_months` of burn. |
| `low` | Anything else. |

With no revenue and no owner contributions, Claude Code's $25 makes the
status `out` in the first month, as the plan expects. Recording Jim's
monthly payment as an `owner:` `[[revenue]]` keeps it `funded`.

## What comes next

Later phases of the revenue plan read these files: the daily usage
workflow runs `cost money` and uploads the JSON, pane serves
`/api/runway` and the public ledger page, and the apps show a banner and
splash when the status is `low` or `out`.
