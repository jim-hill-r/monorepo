# Monorepo Workspace

This workspace contains projects specific to high-level monorepo functionality.

## Projects

### workflow_tests

Tests for GitHub Actions workflows in this repository. See [workflow_tests/README.md](workflow_tests/README.md) for details.

## Structure

This workspace follows the standard Cargo workspace pattern with:
- Top-level `Cargo.toml` defining the workspace
- Member projects in subdirectories
- Shared dependencies defined in `[workspace.dependencies]`
