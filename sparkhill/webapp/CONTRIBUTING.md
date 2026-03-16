# Contributing to sparkhill_webapp

## Getting Started

### Install Toolchain

Use the Cast install command to install all required dependencies:

```bash
cd sparkhill/webapp
cast install
```

This will automatically install:
- Dioxus CLI (`dx`) version 0.7.2
- All required dependencies

### Build

To build the project:

```bash
cargo build
```

To run the development server:

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
