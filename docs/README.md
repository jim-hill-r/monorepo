# auftog

Monorepo for all projects

# Dependencies

- Rust: Recommend using [Rustup](https://www.rust-lang.org/tools/install) and the Rust vscode extension.
- Cargo: Recommend using [Rustup](https://www.rust-lang.org/tools/install) and the Rust vscode extension.

# Bootstrap Repo

- Install dependencies
- Run something similar based on your shell environment

```shell
cd \(projects\)/cast
cargo build --release
cd ../..
cp \(projects\)/cast/target/release/cast cast
```

# Get Started

- To start a work session, run `./cast session start`.

# OLD DOCS: Remove these once irrelevant

# Documentation

- Detailed documentation for this monorepo can be found in `docs`.
- Most commands and scripts will only work in MacOS environment.
- The root level folders are for global shared resources/files.
- The project level folders have specific code for individual projects.

# Getting Started

- Install SurrealDB: Install using [the documentation](https://surrealdb.com/docs/surrealdb/installation), the SurrealDB vscode extension and [Surrealist](https://surrealdb.com/docs/surrealist/installation) admin app.
- Install Perseus: `cargo install perseus-cli`

# Using Cargo (instead of Bazel)

## Build Module with Cargo

- TODO

## Run Module with Cargo

- TODO

## Test Module with Cargo

- TODO

# Using Bazel (instead of Cargo)

- Install Bazel: Recommend using [Bazelisk](https://bazel.build/install/bazelisk) and the Bazel vscode extension.
- Install Buildifer for Bazel: Recommend using [Brew](https://formulae.brew.sh/formula/buildifier)

## Build Module with Bazel

- Navigate to module: Any folder containing a MODULE.bazel can be built independent of the rest of the repo.
- Build using Bazel: Run `bazel build //...`

## Run Module with Bazel

- Navigate to module: Any folder containing a MODULE.bazel can be built independent of the rest of the repo.
- Run `bazel run //...` or `./bazel-bin/bin`

## Test Module with Bazel

- Navigate to module: Any folder containing a MODULE.bazel can be built independent of the rest of the repo.
- Run `bazel test //...`
