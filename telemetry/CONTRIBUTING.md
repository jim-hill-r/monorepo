# Contributing to telemetry

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands"). It
installs Rust with the `wasm32-unknown-unknown` target, Node and wrangler:

```bash
cd telemetry
mise install
```

### Build

To run the Worker locally with `wrangler dev`:

```bash
mise run dev
```

To build the Worker bundle that `wrangler deploy` uploads:

```bash
mise run release
```

Deploys run in CI on pushes to `main` (`mise run deploy`, which needs the
repo's Cloudflare secrets), so there is no need to deploy by hand.

### Test

To run the event tests in `src/event.rs`:

```bash
mise run test
```

To run every CI check (formatting, linting, the wasm build and the tests),
as CI runs them:

```bash
mise run ci
```

## Changing the event format

A new `kind` or field goes in `src/event.rs` with a test that accepts it and
one that refuses a bad value, and in the tables in [README.md](README.md).
Never store anything that identifies a person: the IP address is only a
rate-limit key, and free text is scrubbed and cut before it is written.
