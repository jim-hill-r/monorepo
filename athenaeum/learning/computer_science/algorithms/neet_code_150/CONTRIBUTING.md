# Contributing to neet_code_150

## Getting Started

This crate is its own mise project (it has its own `mise.toml`), so PR CI
builds, lints and tests it whenever a file in this folder changes. The
commands are the repo-wide ones in the root `AGENTS.md` "Dev commands".

### Install Toolchain

```bash
cd athenaeum/learning/computer_science/algorithms/neet_code_150
mise install
```

This installs the stable Rust toolchain with rustfmt and clippy.

### Build and test

```bash
mise run build
mise run test
```

To run everything CI checks (format, clippy with warnings as errors, build
and tests):

```bash
mise run ci
```

`mise run fix` formats the code and applies clippy's safe fixes.

## Adding a problem

- Put each problem in its own file under the module for its pattern
  (`arrays_and_hashing`, `binary_search`, `trees`, ...), and add the module to
  `src/lib.rs` as `pub mod` so it is compiled and linted.
- Start from `src/template.rs`: the problem statement, the recommended
  complexity and the hints as comments, then one function per approach with
  its time and space cost.
- Test every approach against the problem's examples and its edge cases
  (empty input, one element, a miss at either end).
- Don't use `unwrap()` or `expect()`, tests included: the crate's lints warn
  on them and CI fails on warnings.
