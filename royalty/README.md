# royalty

[![Eval coverage](https://raw.githubusercontent.com/jimhillr/monorepo/evals-results/badges/royalty.svg)](https://github.com/jimhillr/monorepo/blob/evals-results/verdicts.json)

Royalty is an open source way for the people who build libraries and
frameworks to be paid when their code is used. A producer puts a
`royalty.toml` in their project that says how they want to be paid: what is
free, what production use costs, and the most anyone can ever be charged. A
consumer runs one command that reads their dependency graph, finds those
files, and works out what is owed under the tiers and caps.

It is named royalty because a royalty is a payment made to a creator each
time their work is used, which is what the project sets out to automate for
code.

Three promises shape the design:

- **Free to start.** Development, tests, CI and staging never cost anything.
- **A ceiling in production.** Producers set caps, consumers set a budget,
  and the lowest one wins.
- **No performance cost.** Usage is measured from the lockfile, git, CI,
  deploys and logs, not from inside the running program.

## Status

Phase 1 is being built. The `royalty_model` crate (`model/`) reads and
validates a `royalty.toml` and works out what a consumer owes; the
`royalty` command (`cli/`) checks a file and scans a consumer's Cargo
dependencies, counting seats from git and deployments from the consumer's
manifests, and writes a report for manual payment. The full
`royalty.toml` format is in the [royalty.toml reference](docs/royalty-toml.md).
The design, the phases and Jim's answers to its decisions are in the
[royalty plan](docs/plans/2026-11-royalty-plan.md). The work items are in
[docs/issues/](docs/issues/README.md).

## Usage

Build the command from this directory with `cargo build --release`; it is
then `target/release/royalty`. It makes no network calls.

### `royalty check [path] [--strict] [--period YYYY-MM]`

Checks one `royalty.toml` (or the one in a directory; the default is the
current directory) and prints each finding with its code and, where one
applies, the requirement it protects:

```text
$ royalty check
error[RTOML-008] ./royalty.toml tier[0].when.environment[0]: "staging": a tier can only name "production"; development, test, ci and staging are always free (ROYL-002)
royalty check: ./royalty.toml: 1 error(s), 0 warning(s)
```

It exits 0 when there are only warnings, 1 on any error (or any warning
with `--strict`), and 2 when the file can't be read. In a consumer's Cargo
project (a `[consumer]` table beside a `Cargo.toml`) it also runs the scan
and fails with RTOML-014 when what is owed breaks the consumer's budget or
policy, naming the dependency and the limit:

```text
$ royalty check --period 2026-11
error[RTOML-014] ./royalty.toml: webframe owes $60.00, over per_producer_period_cents ($40.00); binding "agreement" is never scaled (ROYL-004)
```

Run it in CI so a pull request that would break the budget fails before it
merges. The codes are listed
in [What `royalty check` reports](docs/royalty-toml.md#what-royalty-check-reports).

### `royalty scan [path] [--period YYYY-MM]`

Scans a consumer's Cargo project (the default is the current directory):
it runs `cargo metadata --locked --offline`, so it needs a `Cargo.lock` and
the dependencies' sources on disk (after any `cargo build` or `cargo
fetch`), and it never downloads anything. It reads the `royalty.toml` beside
each dependency's manifest, groups packages into families, and prints what
the consumer's own `royalty.toml` makes them owe this period:

```text
$ royalty scan --period 2026-11
royalty scan: . (organization, production), 2026-11
  chartlib 0.4.2 (direct): team, $25.00 (priced at $70.00), set by the consumer's total budget
  testkit 0.9.0 (transitive): non-production (free), $0.00
  tinycolor 1.0.3 (transitive): not billable (transitive), $0.00
  webframe 2.1.0 (direct): production, $20.00
    packages: webframe@2.1.0, webframe_router@2.1.0
  no terms: plainlib@1.2.0
total: $45.00 of a $45.00 budget
```

`--period` picks the month (the default is the current month, UTC). Seats
are the people who committed to the project's directory in the period,
from `git log`, bots left out; only the count is ever printed. Deployments
are the `[[consumer.deployment]]` manifests whose graph holds the family, or
one for the project when none is listed. A unit the scan can't count is
printed as "not measured" and adds nothing; it is never guessed.

### `royalty report [path] [--period YYYY-MM] [--out dir]`

Does what `royalty scan` does and writes two files (into the consumer's
directory unless `--out` says otherwise):

- `royalty-report.json`, for tools: format version 1, one entry per family
  with its tier, units, every cap and budget step with the amount after it,
  the reason for the final number, and each payee's share and payment
  methods.
- `royalty-report.md`, for people: the same as a table, then how to pay
  each payee, the packages with no terms, and notices.

The same inputs give the same bytes, and the report holds package names,
counts and amounts only: no file paths, no source, and no names or emails
of the consumer's people. The consumer pays by hand from it.

## The file today

Every project in this monorepo already has a `royalty.toml` with a
`[project]` name and description
([configuration.md](../standards/docs/configuration.md)). That is a valid
schema 1 file and means "no payment terms"
([how the existing files map](docs/royalty-toml.md#how-the-existing-files-map)).

## Cost

$0 a month, with no outside service. See [docs/cost.toml](docs/cost.toml)
and "Cost of each phase" in the plan.
