# workflow_tests

This project contains Rust-based tests for GitHub Actions workflows in this repository.

## Purpose

Tests validate that workflow files are correctly configured and contain the necessary logic for:
- CI/CD operations
- Error handling
- Security best practices (e.g., proper quoting of expressions)

## Running Tests

```bash
cd jimhillr
mise run test
```

## Test Coverage

One test file per workflow:

| File | Workflow |
| --- | --- |
| `mise_workflow_tests.rs` | `mise-projects.yml`, `pull-request-ci.yml`, `push-main.yml`, `nightly-ci.yml`, `nightly-cd.yml` |
| `codeql_workflow_tests.rs` | `all-codeql.yml` |
| `delete_branches_workflow_tests.rs` | `all-delete-branches.yml` |
| `agent_skills_workflow_tests.rs` | `pull-request-agent-skills.yml` |
| `pane_redeploy_workflow_tests.rs` | `pane-redeploy.yml` |

## Why Rust Tests?

Rust tests provide several advantages over shell scripts:
- **Type safety**: Catch errors at compile time
- **Better error messages**: Clear, structured test output
- **Maintainability**: Easier to refactor and extend
- **Integration**: Can use Rust libraries for advanced testing
- **Consistency**: Same testing framework as the rest of the codebase
