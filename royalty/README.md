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

Design only. Nothing is built yet. The design, the full `royalty.toml`
format, the phases and the open questions for Jim are in the
[royalty plan](docs/plans/2026-11-royalty-plan.md). The work items are in
[issues/](issues/README.md).

## The file today

Every project in this monorepo already has a `royalty.toml` with a
`[project]` name and description
([configuration.md](../standards/docs/configuration.md)). That is a valid
file in the planned format and means "no payment terms".

## Cost

$0 a month, with no outside service. See [docs/cost.toml](docs/cost.toml)
and "Cost of each phase" in the plan.
