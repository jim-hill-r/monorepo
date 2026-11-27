# royalty

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
`royalty` command (`cli/`) checks a file. Scanning a consumer's
dependencies and writing a report come next. The full
`royalty.toml` format is in the [royalty.toml reference](docs/royalty-toml.md).
The design, the phases and Jim's answers to its decisions are in the
[royalty plan](docs/plans/2026-11-royalty-plan.md). The work items are in
[issues/](issues/README.md).

## Usage

Build the command from this directory with `cargo build --release`; it is
then `target/release/royalty`. It makes no network calls.

### `royalty check [path] [--strict]`

Checks one `royalty.toml` (or the one in a directory; the default is the
current directory) and prints each finding with its code and, where one
applies, the requirement it protects:

```text
$ royalty check
error[RTOML-008] ./royalty.toml tier[0].when.environment[0]: "staging": a tier can only name "production"; development, test, ci and staging are always free (ROYL-002)
royalty check: ./royalty.toml: 1 error(s), 0 warning(s)
```

It exits 0 when there are only warnings, 1 on any error (or any warning
with `--strict`), and 2 when the file can't be read. The codes are listed
in [What `royalty check` reports](docs/royalty-toml.md#what-royalty-check-reports).

## The file today

Every project in this monorepo already has a `royalty.toml` with a
`[project]` name and description
([configuration.md](../standards/docs/configuration.md)). That is a valid
schema 1 file and means "no payment terms"
([how the existing files map](docs/royalty-toml.md#how-the-existing-files-map)).

## Cost

$0 a month, with no outside service. See [docs/cost.toml](docs/cost.toml)
and "Cost of each phase" in the plan.
