# Contributing to homer

## Getting Started

### Install Toolchain

Use the Cast install command to install all required dependencies:

```bash
cd homer
cast install
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
cast ci
```
