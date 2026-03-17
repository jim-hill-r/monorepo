# Contributing to my_lang

## Getting Started

### Install Toolchain

Use the Cast install command to install all required dependencies:

```bash
cd my_lang
cast install
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
cast ci
```

## Notes

- This project requires LLVM 18. Run `cast install` to install it automatically.
- On macOS, you may need to set `LLVM_SYS_180_PREFIX=$(brew --prefix llvm@18)` in your environment.
