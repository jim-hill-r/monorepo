# Contributing to sparkhill_website

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands"):

```bash
cd sparkhill/website
mise install
```

This will automatically install:
- Rust toolchain (rustc, cargo, rustfmt, clippy)

- Perseus CLI 0.4.2, wasm-bindgen and wasm-opt, for the static export

### Build

To build the project:

```bash
cargo build
```

To export the site and serve the static files, rebuilding on changes:

```bash
mise run dev
```

The whole home page is the Perseus template in `src/templates/index.rs`;
`static/` holds only the CSS, images and favicon it links to.

### Test

To run the Rust tests (the template's tests render the page and check its
links, ids and images):

```bash
cargo test
```

To run all CI checks (formatting, linting, build, and tests):

```bash
mise run ci
```
