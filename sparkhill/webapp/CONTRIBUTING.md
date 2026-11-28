# Contributing to sparkhill_webapp

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands"):

```bash
cd sparkhill/webapp
mise install
```

This installs the tools pinned in `mise.toml`:
- Rust stable, with rustfmt, clippy and the `wasm32-unknown-unknown` target
- Dioxus CLI (`dx`) 0.6.x, matching the app's `dioxus` 0.6
- Node 24, for the Playwright end-to-end tests in `e2e/`

### Build

To build the project:

```bash
cargo build
```

To run the development server with hot reload on <http://localhost:8080>:

```bash
mise run dev
```

### Test

To run the Rust unit tests and then the Playwright tests:

```bash
mise run test
```

Use `cargo test` for only the Rust tests.

To run all CI checks (formatting, linting, build, and tests):

```bash
mise run ci
```
