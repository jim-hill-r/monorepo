# Contributing to cookbook_data_md

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands"):

```bash
cd cookbook/data_md
mise install
```

This will automatically install:
- Rust toolchain (rustc, cargo, rustfmt, clippy)

### Build

To build the project:

```bash
cargo build
```

### Test

To run Rust unit tests:

```bash
cargo test
```

To run all CI checks (formatting, linting, build, and tests):

```bash
mise run ci
```
