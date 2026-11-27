# Contributing to web

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands"),
from this folder. It installs Rust with the `wasm32-unknown-unknown`
target, the Dioxus CLI (`dx`) and Node:

```bash
cd cahokia/web
mise install
npm ci
npx playwright install --with-deps chromium
```

### Build

To serve the app with hot reload at `http://localhost:8080`:

```bash
mise run dev
```

To build the release bundle into `dist/`:

```bash
mise run release
```

Sign-in only works with the `local.app.cahokia.com:8080` callback URL, so
point that host name at `127.0.0.1` in your hosts file to try it locally.

### Test

To run the Rust tests and then the Playwright tests in `tests/` (which
start `dx serve` themselves):

```bash
mise run test
```

To run every CI check for this app (formatting, linting, build, Rust and
Playwright tests):

```bash
mise run ci
```

[tests/README.md](tests/README.md) explains the Playwright tests and how
to run one file or debug one.

### Where code goes

- Put pages and components that desktop and mobile also show in `ui`; keep
  only web routes, the sidebar and web-only views here.
- Add a Playwright test in `tests/` for every UI change.
- Deploys go through the [cloudflare](../cloudflare/README.md) crate.
