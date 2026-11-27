# Contributing to tofu_state

## Getting Started

Install the toolchain (Rust with the wasm target, `worker-build`, wrangler
and OpenTofu, pinned in `mise.toml`):

```bash
mise install
```

Run every CI check, including the end-to-end test against `wrangler dev`:

```bash
mise run ci
```

`cargo test` runs the protocol tests natively and needs no Workers tooling.
For a local Worker, `mise run dev` serves it with a local D1; put test
passwords (never the real ones) in `.dev.vars`, which git ignores.
