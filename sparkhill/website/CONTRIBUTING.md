# Contributing to sparkhill_website

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands"):

```bash
cd sparkhill/website
mise install
```

This will automatically install:
- Rust toolchain (rustc, cargo, rustfmt, clippy)

Perseus CLI must be installed separately:

```bash
cargo install perseus-cli
```

### Build

To build the project:

```bash
cargo build
```

To run the development server:

```bash
perseus serve -w
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
