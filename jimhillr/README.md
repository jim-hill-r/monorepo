# jimhillr

[![Eval coverage](https://raw.githubusercontent.com/jimhillr/monorepo/evals-results/badges/monorepo.svg)](https://github.com/jimhillr/monorepo/blob/evals-results/verdicts.json)

A Cargo workspace for code about the repository itself rather than any one
project. Today that is [workflow_tests](workflow_tests/README.md), which
checks the GitHub Actions workflows in `.github/workflows/`. How to build
and test it is in [CONTRIBUTING.md](CONTRIBUTING.md).

## Projects

### workflow_tests

Tests for GitHub Actions workflows in this repository. See [workflow_tests/README.md](workflow_tests/README.md) for details.

## Structure

This workspace follows the standard Cargo workspace pattern with:
- Top-level `Cargo.toml` defining the workspace
- Member projects in subdirectories
- Shared dependencies defined in `[workspace.dependencies]`
