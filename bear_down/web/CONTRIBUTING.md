# Contributing to bear_down/web

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands"),
from this folder. It installs Rust with the `wasm32-unknown-unknown`
target, the Dioxus CLI (`dx`) and Node:

```bash
cd bear_down/web
mise install
npm ci
npx playwright install --with-deps chromium
```

When you change the `dioxus` version in `../Cargo.lock`, change
`cargo:dioxus-cli` in `mise.toml` to match.

### Build

To serve the page with hot reload:

```bash
mise run dev
```

The page reads `public/rooting.json`. To refresh it with this week's
simulation, run the CLI from the workspace root:
`cargo run -p bear_down_cli -- --out web/public/rooting.json`.

To build the release bundle:

```bash
mise run release
```

### Test

To run the Rust tests and then the Playwright tests in `tests/`:

```bash
mise run test
```

To run every CI check for this page (formatting, linting, build, Rust and
Playwright tests):

```bash
mise run ci
```

Add a Playwright test in `tests/` for every UI change.
