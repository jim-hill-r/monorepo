# Contributing to royalty

## Getting Started

Royalty is a docs project today: a plan and issue files, with no code. The
commands below are the standard ones from the root
[AGENTS.md](../AGENTS.md) "Dev commands". They do nothing yet, and will
build and test the Rust crates once Phase 1 of the
[plan](docs/plans/2026-11-royalty-plan.md) adds them.

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

Check the repo standards, which include the issue files in `issues/`:

```bash
mise run //:audit
```

## Changing the design

The `royalty.toml` format and the design rules are in the
[plan](docs/plans/2026-11-royalty-plan.md). A change to the format updates
the plan (or, once it exists, `docs/royalty-toml.md`) in the same PR. A
change that answers or reopens one of the plan's "Decisions for Jim" is
labelled `needs-human`.

## Working an issue

Issues are one file each in [issues/](issues/README.md), in the format of
[issue-management.md](../standards/docs/issue-management.md). An issue that
names a decision in its "Blocked on" line waits until Jim has answered that
decision in the plan.
