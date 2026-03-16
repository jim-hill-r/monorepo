# Contributing to sparkhill_website

## Getting Started

### Install Toolchain

Use the Cast install command to install all required dependencies:

```bash
cd sparkhill/website
cast install
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
cast ci
```
