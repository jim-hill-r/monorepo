# Contributing to controlled_chaos

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands"):

```bash
cd controlled_chaos
mise install
```

This will automatically install:
- Rust toolchain (rustc, cargo, rustfmt, clippy)

### Build

To build the project:

```bash
cargo build
```

To build a release binary:

```bash
cargo build --release
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
