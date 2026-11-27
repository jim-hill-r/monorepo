# Contributing to cost

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands"):

```bash
cd cost
mise install
```

### Build

```bash
mise run build
```

### Test

```bash
mise run test   # unit tests for both crates
mise run ci     # fmt, clippy, build and tests, as CI runs them
```

### Run

```bash
cargo run -p cost_cli -- check
```

## Adding a rule

Put the rule in `model/src/rules.rs` as a pure function over parsed files,
give it a `Code` in `model/src/finding.rs`, add a unit test that shows it
firing and not firing, and add it to the table in
`standards/docs/cost.md`.
