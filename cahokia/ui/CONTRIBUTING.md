# Contributing to ui

## Getting Started

### Install Toolchain

Use the Cast install command to install all required dependencies:

```bash
cd cahokia/ui
cast install
```

### Build

To build the project:

```bash
cargo build
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

`ui` is in the cahokia workspace's `default-members`, so it is also built and tested by `cargo build` and `cargo test` run from `cahokia/`.

### Adding a component

Put each component in its own module under `src/`, re-export it from `src/lib.rs`, and keep its styles in `assets/styling/<component>.css`, loaded with `asset!`.
