# Contributing to my_lang

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands"):

```bash
cd my_lang
mise install
```

This will automatically install:
- Rust toolchain (rustc, cargo, rustfmt, clippy)
- LLVM 18 (required for code generation via inkwell)

### Build

To build the project:

```bash
cargo build
```

### Test

To run Rust unit tests:

```bash
cargo test
```

To run all CI checks (formatting, linting, build, and tests):

```bash
mise run ci
```

## Notes

- This project requires LLVM 18. On Linux, `mise run ci:deps` installs it in CI (apt); install it locally from your package manager.
- On macOS, you may need to set `LLVM_SYS_180_PREFIX=$(brew --prefix llvm@18)` in your environment.
