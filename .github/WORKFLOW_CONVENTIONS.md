# GitHub Workflows Conventions

This document describes conventions and best practices for GitHub workflows in this repository.

## General Principles

### Minimal Logic in Workflows

**IMPORTANT**: GitHub workflows should have as little logic as possible. They should primarily run `cast` CLI commands. If complex logic is required, it should be added to the `cast` CLI project, not hardcoded in workflow YAML files.

#### Why Keep Workflows Minimal?

1. **Testability**: Code in `cast` CLI can be unit tested, whereas workflow YAML logic cannot
2. **Reusability**: Logic in `cast` CLI can be used locally and in CI, not just in workflows
3. **Maintainability**: Rust code is easier to maintain and refactor than bash scripts in YAML
4. **Type Safety**: Rust provides compile-time guarantees that bash scripts don't
5. **Debugging**: Easier to debug Rust code locally than workflow YAML

#### What Belongs in Workflows vs Cast CLI?

**Workflows should contain:**
- Workflow triggers and event configuration
- Environment setup (checkout, toolchain installation)
- Simple glue code to call `cast` commands
- Setting environment variables from GitHub context
- Basic conditionals for workflow control

**Cast CLI should contain:**
- Business logic (git operations, file parsing, validation)
- Data transformations (JSON parsing, filtering)
- Complex conditionals and loops
- Error handling and validation
- Any logic that could be reused outside of CI

#### Examples

**❌ Bad - Too much logic in workflow:**
```yaml
- name: Find changed projects
  run: |
    # SHA validation with regex
    if ! [[ "$BASE_SHA" =~ ^[0-9a-f]{40}$ ]]; then
      echo "Invalid SHA"
      exit 1
    fi
    
    # Complex git operations
    git diff --name-only $BASE_SHA $HEAD_SHA | while read file; do
      # Walk up directory tree
      dir=$(dirname "$file")
      while [ "$dir" != "." ]; do
        if [ -f "$dir/Cast.toml" ]; then
          echo "$dir"
          break
        fi
        dir=$(dirname "$dir")
      done
    done | sort -u
```

**✅ Good - Minimal workflow, logic in cast CLI:**
```yaml
- name: Find changed projects
  run: |
    cast project with-changes --base $BASE_SHA --head $HEAD_SHA
```

#### Migration Strategy

If you find complex logic in a workflow:
1. Create a new `cast` CLI command or subcommand
2. Move the logic to the `cast` project in Rust
3. Add unit tests for the new functionality
4. Update the workflow to call the new `cast` command
5. Remove the old bash logic from the workflow

## Best Practices

### Quoting GitHub Actions Expressions in Bash

When assigning GitHub Actions expressions to bash variables, always wrap them in double quotes to prevent errors:

**❌ Bad - Unquoted expression:**
```yaml
- run: |
    BASE_SHA=${{ github.event.pull_request.base.sha }}
```

**✅ Good - Quoted expression:**
```yaml
- run: |
    BASE_SHA="${{ github.event.pull_request.base.sha }}"
```

Why quote expressions?
- Prevents bash errors when the expression evaluates to empty string
- Handles special characters safely
- Makes variable assignment more robust in edge cases

### Concurrency Control
Some workflows require concurrency control to prevent multiple instances from running simultaneously:

- **start-a-new-task.yml**: Implements concurrency control by checking for open PRs created by the Copilot user before starting a new agent task. This prevents multiple agent tasks from running at the same time.

### Agent Task Workflows

#### Preventing Concurrent Agent Tasks
The `start-a-new-task.yml` workflow ensures only one agent task runs at a time by:
1. Checking for open PRs created by Copilot bot users ('Copilot' and 'copilot-swe-agent[bot]') using `gh pr list --state open --author <bot-name>`
2. Skipping task creation if any active agent tasks exist (open PRs from either bot)
3. Using conditional step execution based on the check result

This pattern can be applied to other automated task workflows that need to prevent concurrent execution.

#### Using agent-copilot Binary
The `agent-copilot` binary is used to create GitHub Copilot agent tasks programmatically. It:
- Lives in `agent-copilot/artifacts/x86_64-unknown-linux-gnu/agent-copilot`
- Calls the GitHub Copilot API directly (same as `gh agent-task create`)
- Requires a GitHub token with `copilot` scope

## Authentication

### GITHUB_TOKEN vs Custom PAT
- **GITHUB_TOKEN**: Use for read operations like checking PR status
- **Custom PAT (e.g., START_NEW_AI_AGENT_TASK_WORKFLOW_PAT)**: Use for operations requiring elevated permissions like creating agent tasks

## Testing Workflows

All workflows should have corresponding Rust tests in the `workflow_tests` project:
- `cast_ci_workflow_tests.rs`: Tests for the cast CI workflow
- `start_a_new_task_workflow_tests.rs`: Tests for the agent task workflow

Test suites validate:
- File existence
- YAML syntax
- Workflow configuration
- Required permissions
- Logic correctness
- Proper quoting of expressions

Run tests before committing workflow changes:

```bash
cd workflow_tests
cargo test
```
