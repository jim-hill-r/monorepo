# Contributing to sparkhill/cloudflare

This project only deploys the sparkhill website to Cloudflare Pages. Change the app itself in
[`sparkhill/website`](../website/CONTRIBUTING.md); change this project's wrangler and mise
config, or its config tests, here.

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands"). It
installs Rust, Node and wrangler:

```bash
cd sparkhill/cloudflare
mise install
```

### Build

To build the static files this project deploys:

```bash
mise run release
```

Deploys run in CI on pushes to `main` (`mise run deploy`, which needs the
repo's Cloudflare secrets), so there is no need to deploy by hand.

### Test

To run the config tests in `tests/`, with formatting and linting, as CI
runs them:

```bash
mise run ci
```
