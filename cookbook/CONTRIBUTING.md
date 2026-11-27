# Contributing to cookbook

cookbook is a Cargo workspace. This file covers the workspace as a whole; each
crate has its own CONTRIBUTING.md for what is specific to it ([web](web/CONTRIBUTING.md), [core](core/CONTRIBUTING.md), [data_md](data_md/CONTRIBUTING.md), [recipe_gen](recipe_gen/CONTRIBUTING.md) and [cloudflare](cloudflare/CONTRIBUTING.md)).

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands"). The
workspace's `mise.toml` installs Rust with rustfmt and clippy:

```bash
cd cookbook
mise install
```

The web app needs more (the Dioxus CLI, Node and Playwright): run
`mise install` in `web/` too, as its CONTRIBUTING.md says.

### Build

To build every crate in the workspace:

```bash
mise run build
```

### Test

To run every crate's tests:

```bash
mise run test
```

To run every CI check for the workspace (formatting, linting, build and
tests), as CI runs them:

```bash
mise run ci
```
