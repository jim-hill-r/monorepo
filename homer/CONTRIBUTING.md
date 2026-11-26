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

Tests never call the real Zillow API. `ZillowClient::with_base_url` points the
client at a local [mockito](https://docs.rs/mockito) server instead, so HTTP
handling (success, error statuses, bad JSON) is tested offline.

To run all CI checks (formatting, linting, build, and tests):

```bash
cast ci
```
