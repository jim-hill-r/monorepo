# Contributing to homer

## Code layout

`src/main.rs` only reads the command line. The work lives in the library
(`src/lib.rs`), and each module keeps its unit tests beside its code:

- `zillow.rs`: the RapidAPI client and `parse_property_details`.
- `evaluate.rs`: `HomePreferences` and `evaluate_property`.
- `report.rs`: the text report and `--json` output.
- `quota.rs`: the guard that keeps homer inside the RapidAPI free plan.

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

`tests/cli.rs` runs the real `homer` binary against the
[trycmd](https://docs.rs/trycmd) transcripts in `tests/cmd/`: `--help`, a
lookup, a scored lookup, an address Zillow has no property for and a missing
API key. Each case is a `.toml` file
with the arguments, plus `.stdout` and `.stderr` files with the expected
output. The binary reaches a local mockito server through the hidden
`--base-url` flag (`HOMER_BASE_URL`), so these are offline too. After an
intended output change, update the transcripts and review the diff:

```bash
TRYCMD=overwrite cargo test --test cli
```

To run all CI checks (formatting, linting, build, and tests):

```bash
mise run ci
```
