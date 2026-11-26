# Contributing to homer

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands"):

```bash
cd homer
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

Tests never call the real Zillow API. `ZillowClient::with_base_url` points the
client at a local [mockito](https://docs.rs/mockito) server instead, so HTTP
handling (success, error statuses, bad JSON) is tested offline.

To run all CI checks (formatting, linting, build, and tests):

```bash
mise run ci
```
