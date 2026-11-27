# Contributing to chores

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands"):

```bash
cd chores
mise install
```

### Build and test

```bash
mise run test   # unit tests
mise run ci     # fmt, clippy, build and tests, as CI runs them
```

### Run

```bash
cargo run -- next-task
cargo run -- --help
```

Every command is a pure function over files in a folder, so tests build a
small repo in a temporary directory and check the output exactly.
