# Contributing to core

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands").
`core` is a member of the cahokia Cargo workspace, so its tasks run from
`cahokia/`:

```bash
cd cahokia
mise install
```

### Build

To build the crate:

```bash
cargo build -p core
```

### Test

To run its tests:

```bash
cargo test -p core
```

To run all CI checks for the cahokia workspace (formatting, linting, build,
and tests):

```bash
mise run ci
```

`core` is in the workspace's `default-members`, so a plain `cargo build` or
`cargo test` in `cahokia/` covers it too.

### Where code goes

- Keep `core` free of UI and platform code: anything every renderer needs
  (layout, coordinates, tree queries) belongs here, and drawing belongs in
  `ui`.
- Add tests next to the code in a `#[cfg(test)] mod tests`, and update the
  README's usage examples when a public API changes.
- Results must be deterministic: the same tree always gives the same layout.
