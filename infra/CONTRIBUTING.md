# Contributing to infra

## Getting Started

Install the toolchain (Rust and OpenTofu, pinned in `mise.toml`):

```bash
mise install
```

Build, test and run every CI check:

```bash
mise run build
mise run test
mise run ci
```

After changing a construct or stack, regenerate the committed output with
`mise run synth`. See [README.md](README.md) for adding resource types.
