# Example royalty reports

Two runs of `royalty report` for November 2026, committed so the format can
be read without building anything.

| Example | What it is | Owed |
| --- | --- | --- |
| [acme_shop](acme_shop/royalty-report.md) ([JSON](acme_shop/royalty-report.json)) | The fixture consumer in `cli/tests/fixtures/acme_shop`: Acme's shop from the [royalty.toml reference](../royalty-toml.md#what-acme-owes-in-one-month), with path dependencies on the reference's example producers and 14 people committing. | $45.00 |
| [cost](cost/royalty-report.md) ([JSON](cost/royalty-report.json)) | This monorepo's `cost` project, a real Rust workspace. It has no `[consumer]` table, so it is a person in development, and none of its crates.io dependencies ships a `royalty.toml`. | $0.00 |

The Acme report is checked by a test
(`the_committed_acme_example_report_is_current`): a change to the report
format fails until the file is regenerated with
`ROYALTY_BLESS=1 cargo test`. The cost report is a snapshot of one day's
`Cargo.lock`; regenerate it from the repo root with
`royalty report cost --period 2026-11 --out royalty/docs/examples/cost`.

## Reading the Acme report

**The total line** says what Acme pays this month, $45.00, and the budget it
was held to: `[consumer.budget] period_cents = 4500` in Acme's
`royalty.toml`. If the total could not be held there, the line would say
"Over budget", and `royalty check` would fail in Acme's CI.

**One row per family.** A family is every package that shares a producer's
`family` and payees, so `webframe` and `webframe_router` are one row,
billed once.

- `chartlib` asks $5 a seat. Git shows 14 people committed to Acme's project
  in November, so it is priced at $70.00 ("Before caps"). Its producer caps
  one project at $40.00 ("After producer caps"). It is `voluntary`, so
  Acme's $45 budget scales it to the $25.00 left after the fixed amounts:
  "Set by the consumer's total budget".
- `webframe` is an `agreement`: $20 a production deployment after the first.
  Acme lists two deployments whose manifests both pull it in, so it is
  $20.00, and an agreement amount is never scaled. The note says its 90-day
  trial isn't applied yet: Phase 1 has no payments ledger to know when
  Acme's production use started.
- `tinycolor` is used only through `webframe`, and its producer asks to be
  paid only by direct users (`reach = "direct"`), so it is "not billable".
- `testkit` is only a dev-dependency: non-production use is always free,
  whatever its terms say.

**How to pay** lists each amount and every payee's share, with their first
payment method. Royalty never pays anyone; Acme does, by hand.

**No payment terms** lists packages with no `royalty.toml`, or with one that
doesn't apply. **Notices** explain anything unusual: here, `copycat` ships a
`royalty.toml` written for another package, which the scanner ignores so a
copied file can't put terms on someone else's code.

The JSON file has the same facts for tools, plus each price line's exact
amount in millionths of a cent and every cap step in order. Neither file
holds a file path, a commit email or the name of anyone at Acme: only
package names, counts and amounts.
