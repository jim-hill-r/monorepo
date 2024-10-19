# auftog

Monorepo for all projects

# Documentation

- Detailed documentation for this monorepo can be found in `~docs`.

# Getting Started

- Install Bazel: Recommend using [Bazelisk](https://bazel.build/install/bazelisk) and the Bazel vscode extension.
- Install Rust: Recommend using [Rustup](https://www.rust-lang.org/tools/install) and the Rust vscode extension.
- Install Perseus: `cargo install perseus-cli`

# Build Module with Bazel

- Navigate to module: Any folder containing a MODULE.bazel can be built independent of the rest of the repo.
- Build using Bazel: Run `bazel build //...`

# Run Module with Bazel

- Navigate to module: Any folder containing a MODULE.bazel can be built independent of the rest of the repo.
- Run `bazel run //...` or `./bazel-bin/bin`

# Test Module with Bazel

- Navigate to module: Any folder containing a MODULE.bazel can be built independent of the rest of the repo.
- Run `bazel test //...`
