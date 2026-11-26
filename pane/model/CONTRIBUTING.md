# Contributing to pane_model

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands"):

```bash
cd pane/model
mise install
```

### Build

```bash
cargo build
```

### Test

```bash
cargo test
```

To run all CI checks (formatting, linting, build, and tests):

```bash
mise run ci
```

### Changing the types

Keep the crate free of anything that won't compile to `wasm32-unknown-unknown`
(no file system or network access), because `pane_web` uses it in the
browser. Bump `SNAPSHOT_VERSION` for any change that breaks older JSON.
