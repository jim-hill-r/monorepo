# cast_cli

The CLI binary for the Cast tool — highly opinionated tooling for Rust monorepos.

`cast_cli` provides the `cast` command-line interface that delegates to `cast_core` for all functionality. It is the entry point for running Cast commands such as `cast build`, `cast test`, `cast ci`, `cast install`, and more.

A cast is a [group of crabs](https://www.originaldiving.com/blog/our-favourite-collective-nouns-for-sea-creatures#:~:text=A%20group%20of%20crabs%20is,crabs%20dominating%20access%20to%20food.).

# Dependencies

- Rust
- Cargo

# Build

```bash
cargo build --release
```

# Install

To install the `cast` binary locally:

```bash
cargo install --path .
```

# Usage

Run `cast -h` to see all available commands:

```bash
cast -h
```

For full documentation of Cast features, see the [cast_core README](../core/README.md).
