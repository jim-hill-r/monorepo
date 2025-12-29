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

### Cast CI Workflow Tests (`cast_ci_workflow_tests.rs`)

Tests for `.github/workflows/cast-ci.yml`:

- **File and Structure Tests**
  - Workflow file exists
  - YAML syntax is valid
  - Workflow can be parsed

- **Trigger and Configuration Tests**
  - Pull request trigger is configured
  - Uses cast CLI to detect changes
  - Searches for Cast.toml files
  - Builds cast CLI
  - Runs cast ci command
  - Sets up Rust toolchain
  - Handles no projects changed

- **Error Handling Tests**
  - Contains explicit git fetch commands
  - Checks git diff exit code
  - Captures stderr from cast command
  - Prints error output on failure
  - Exits with error on cast command failure
  - Fetch commands use graceful failure (`|| true`)

- **Security Tests**
  - BASE_SHA is properly quoted
  - HEAD_SHA is properly quoted

- **Toolchain Tests**
  - Workflow uses `cast toolchain install` instead of manual tool installation
  - Workflow does not manually install Dioxus CLI
  - Workflow does not manually install Playwright
  - Workflow installs rustfmt component
  - Workflow installs clippy component
  - Workflow installs toolchain before running CI

- **Integration Tests**
  - `cast toolchain check` works on pure Rust library projects
  - `cast toolchain check` detects Dioxus framework requirements
  - `cast toolchain list` command works correctly
  - `cast toolchain install --dry-run` shows installation status

## Why Rust Tests?

Rust tests provide several advantages over shell scripts:
- **Type safety**: Catch errors at compile time
- **Better error messages**: Clear, structured test output
- **Maintainability**: Easier to refactor and extend
- **Integration**: Can use Rust libraries for advanced testing
- **Consistency**: Same testing framework as the rest of the codebase
