# Contributing to evals

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands"):

```bash
cd evals
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
cargo run -p evals_cli -- check
```

## Notes

- Keep `model` free of I/O: callers read files and pass text in.
- A change to a verdict rule changes what pane shows, so update the rule
  table at the top of `model/src/verdict.rs` and the plan's Verdicts
  section with it.
