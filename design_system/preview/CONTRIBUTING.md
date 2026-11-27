# Contributing to design_system/preview

The page is built by `src/main.rs` from `../tokens` (through
`design_system_model`), `../dist`, `../js/theme.js` and `../dioxus/assets/design-system-fonts`.
Its own layout is `src/preview.css`, which uses only `--ds-` tokens so each
panel looks the way an app in that theme would.

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands"). It
installs Rust, Node and wrangler:

```bash
cd design_system/preview
mise install
```

### Build

To build the preview site into `dist/`:

```bash
mise run release
```

To build it and serve it on <http://localhost:8000>:

```bash
mise run dev
```

### Test

To run every CI check (formatting, linting, build and tests), as CI runs
them, before committing:

```bash
mise run ci
```
