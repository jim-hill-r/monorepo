# Contributing to bear_down/cloudflare

This project only deploys the bear_down web app to Cloudflare Pages. Change the app itself in
[`bear_down/web`](../web/CONTRIBUTING.md); change this project's wrangler and mise
config, or its config tests, here.

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands"). It
installs Rust, Node and wrangler:

```bash
cd bear_down/cloudflare
mise install
```

### Build

To run the season simulation and build the static files this project
deploys (the CLI downloads the nflverse schedule, so it needs the network):

```bash
mise run release
```

Deploys run in CI on pushes to `main` (`mise run deploy`, which needs the
repo's Cloudflare secrets), so there is no need to deploy by hand.

### Test

To run the config tests in `src/lib.rs`, with formatting and linting, as CI
runs them:

```bash
mise run ci
```
