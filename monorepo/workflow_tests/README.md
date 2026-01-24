# Workflow Tests

This project contains Rust-based tests for GitHub Actions workflows in this repository.

## Purpose

Tests validate that workflow files are correctly configured and contain the necessary logic for:
- CI/CD operations
- Error handling
- Security best practices (e.g., proper quoting of expressions)

## Running Tests

```bash
cd workflow_tests
cargo test
```

## Test Coverage

### CI Workflow Tests (`cast_ci_workflow_tests.rs`)

Tests for `.github/workflows/pull-request-ci.yml` and `.github/workflows/trunk-ci.yml`:

- **Pull Request CI Tests**
  - Workflow file exists
  - YAML syntax is valid
  - Workflow can be parsed
  - Pull request trigger is configured
  - Uses `--check` flag for validation-only mode
  - Uses `--only-changed` to check only projects with changes
  - Uses `--recursive 2` to find all Cast projects

- **Trunk CI Tests**
  - Workflow file exists
  - YAML syntax is valid
  - Workflow can be parsed
  - Push to main trigger is configured
  - Uses `--release` flag to build artifacts
  - Uses `--only-changed` to check only projects with changes
  - Uses `--recursive 2` to find all Cast projects
  - Ignores changes to artifacts directories

- **Shared Tests**
  - Builds cast CLI
  - Runs cast ci command
  - Sets up Rust toolchain
  - Handles no projects changed

- **Security Tests**
  - BASE_SHA is properly quoted
  - HEAD_SHA is properly quoted

- **Toolchain Tests**
  - Workflow uses `cast install` instead of manual tool installation
  - Workflow does not manually install Dioxus CLI
  - Workflow does not manually install Playwright
  - Workflow installs rustfmt component
  - Workflow installs clippy component
  - Workflow installs toolchain before running CI

- **Integration Tests**
  - `cast install check` works on pure Rust library projects
  - `cast install check` detects Dioxus framework requirements
  - `cast install list` command works correctly
  - `cast install --dry-run` shows installation status

## Why Rust Tests?

Rust tests provide several advantages over shell scripts:
- **Type safety**: Catch errors at compile time
- **Better error messages**: Clear, structured test output
- **Maintainability**: Easier to refactor and extend
- **Integration**: Can use Rust libraries for advanced testing
- **Consistency**: Same testing framework as the rest of the codebase
