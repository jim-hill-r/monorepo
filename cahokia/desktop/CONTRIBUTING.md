# Contributing to desktop

## Getting Started

### Install Toolchain

Use the Cast install command to install all required dependencies (Rust toolchain and the Dioxus CLI):

```bash
cd cahokia/desktop
cast install
```

Building the desktop app on Linux also requires the system WebView/GTK libraries used by Dioxus desktop (for example `libwebkit2gtk-4.1-dev` and `libgtk-3-dev`).

### Build

To build the project:

```bash
cargo build
```

To run the app during development:

```bash
dx serve
```

### Test

To run Rust unit tests:

```bash
cargo test
```

To run all CI checks (formatting, linting, build, and tests):

```bash
cast ci
```

Note: `desktop` is not in the cahokia workspace's `default-members`, so workspace-level builds from `cahokia/` skip it; run commands from this directory.
