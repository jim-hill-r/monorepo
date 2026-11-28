# Contributing to royalty

## Getting Started

Royalty is a Rust workspace. `model/` is the `royalty_model` crate: the
schema 1 types, the parser and validation, with no I/O. The commands below
are the standard ones from the root [AGENTS.md](../AGENTS.md) "Dev
commands".

### Install Toolchain

Install [mise](https://mise.jdx.dev), then set up the git hooks once per
clone and install the project's tools:

```bash
mise run //:hooks
cd royalty
mise install
```

### Build

```bash
mise run build
```

### Test

```bash
mise run test   # the project's tests
mise run ci     # everything CI runs for this project
```

The model's tests parse every `royalty.toml` in the monorepo and every
`toml` block in [docs/royalty-toml.md](docs/royalty-toml.md), so a change to
the reference's examples is tested too. Tests that check a requirement
start with its ID in snake case (`royl_007_...`) and are listed in
[evals.toml](evals.toml).

Check the repo standards, which include the issue files in `docs/issues/`:

```bash
mise run //:audit
```

## Changing the design

The `royalty.toml` format is in the
[reference](docs/royalty-toml.md), and the design rules and decisions are in
the [plan](docs/plans/2026-11-royalty-plan.md). A change to the format
updates the reference in the same PR. A change that alters what a consumer
owes, or answers or reopens one of the plan's "Decisions for Jim", is
labelled `needs-human`.

## Working an issue

Issues are one file each in [docs/issues/](docs/issues/README.md), in the format of
[issue-management.md](../standards/docs/issue-management.md). An issue with a
"Blocked on" or "Waits on" line waits for what it names.
