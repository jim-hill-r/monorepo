# Royalty report, 2026-11

Consumer: `github:acme-example`, organization, commercial, stage production. Worked out by `royalty report` from the dependency graph, git history and the consumer's `royalty.toml`; the rules are in the royalty.toml reference.

**Total owed: $45.00** of a $45.00 budget.

## Families

| Family | Version | Dependency | Tier | Units | Before caps | After producer caps | Owed | Why |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `chartlib` | 0.4.2 | direct | team | 14 seat | $70.00 | $40.00 | $25.00 | Set by the consumer's total budget. |
| `testkit` | 0.9.0 | transitive | non-production (free) |  | $0.00 | $0.00 | $0.00 |  |
| `tinycolor` | 1.0.3 | transitive | not billable (transitive) |  | $0.00 | $0.00 | $0.00 |  |
| `webframe` | 2.1.0 | direct | production | 2 deployment | $20.00 | $20.00 | $20.00 | Binding `agreement`: never scaled. Note: the producer offers a 90-day trial from the first production use, which this report doesn't apply yet (it needs the payments ledger); owe nothing if the trial hasn't ended. |

## How to pay

- `chartlib` (voluntary, report mode): $25.00
  - Chartlib maintainers: $25.00 by github-sponsors: https://github.com/sponsors/chartlib-example
- `webframe` (agreement, report mode): $20.00
  - Webframe maintainers: $20.00 by github-sponsors: https://github.com/sponsors/webframe-example

## No payment terms

`copycat@0.1.0`, `plainlib@1.2.0`

## Notices

- `copycat@0.1.0`: its royalty.toml is for ["chartlib"], not this package; ignored
