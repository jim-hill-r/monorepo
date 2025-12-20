# GitHub Workflows Conventions

This document describes conventions and best practices for GitHub workflows in this repository.

## General Principles

### Minimal Logic in Workflows
GitHub workflows should have as little logic as possible. They should primarily run `cast` CLI commands. If complex logic is required, it should be added to the `cast` CLI project, not hardcoded in workflow YAML files.

### Concurrency Control
Some workflows require concurrency control to prevent multiple instances from running simultaneously:

- **start-a-new-task.yml**: Implements concurrency control by checking for open PRs created by the Copilot user before starting a new agent task. This prevents multiple agent tasks from running at the same time.

### Agent Task Workflows

#### Preventing Concurrent Agent Tasks
The `start-a-new-task.yml` workflow ensures only one agent task runs at a time by:
1. Checking for open PRs created by the 'Copilot' user using `gh pr list --state open --author Copilot`
2. Skipping task creation if any active agent tasks exist (open PRs from Copilot)
3. Using conditional step execution based on the check result

This pattern can be applied to other automated task workflows that need to prevent concurrent execution.

#### Using agent-copilot Binary
The `agent-copilot` binary is used to create GitHub Copilot agent tasks programmatically. It:
- Lives in `projects/agent-copilot/artifacts/x86_64-unknown-linux-gnu/agent-copilot`
- Calls the GitHub Copilot API directly (same as `gh agent-task create`)
- Requires a GitHub token with `copilot` scope

## Authentication

### GITHUB_TOKEN vs Custom PAT
- **GITHUB_TOKEN**: Use for read operations like checking PR status
- **Custom PAT (e.g., START_NEW_AI_AGENT_TASK_WORKFLOW_PAT)**: Use for operations requiring elevated permissions like creating agent tasks

## Testing Workflows

All workflows should have corresponding test scripts in `.github/workflows/`:
- `test-start-a-new-task.sh`: Tests for the agent task workflow
- `test-cast-ci.sh`: Tests for the cast CI workflow
- `test-cast-ci-error-handling.sh`: Tests error handling in cast CI workflow

Test scripts validate:
- File existence
- YAML syntax
- Workflow configuration
- Required permissions
- Logic correctness

Run tests before committing workflow changes.
