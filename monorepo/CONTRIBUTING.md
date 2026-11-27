# Contributing to monorepo

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands"). It
installs Rust with rustfmt and clippy:

```bash
cd monorepo
mise install
```

### Build

```bash
mise run build
```

### Test

To run the workflow tests:

```bash
mise run test
```

To run every CI check (formatting, linting, build and tests), as CI runs
them:

```bash
mise run ci
```

## Adding a workflow test

When you add or change a workflow in `.github/workflows/`, add or update its
test file in `workflow_tests/tests/` (one file per workflow) and the table in
[workflow_tests/README.md](workflow_tests/README.md).
