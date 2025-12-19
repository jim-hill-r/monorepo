# GitHub Actions Workflows

## start-a-new-task.yml

This workflow automatically creates a GitHub Copilot agent task after a PR created by the Copilot agent is merged.

### How It Works

1. **Trigger**: The workflow runs when a pull request is closed and merged, but only if it was created by the GitHub Copilot agent (`user.login == 'Copilot'`).

2. **Agent Task Creation**: The workflow uses the `agent-copilot` binary to create a new GitHub Copilot agent task with:
   - Title: "Start a new task"
   - Problem Statement: Content from `.github/agent-prompts/start-a-new-task.md`
   - Repository: The current repository
   - Note: This directly creates an agent task using the GitHub Copilot API, bypassing the need to create an issue first.

3. **Authentication**: Uses the standard `GITHUB_TOKEN` provided by GitHub Actions, which has the necessary permissions for creating agent tasks via the Copilot API.

### Setup Requirements

The workflow requires:
1. The `agent-copilot` binary must be present at `projects/agent-copilot/artifacts/agent-copilot`
2. The `GITHUB_TOKEN` provided by GitHub Actions (automatically available)

### Permissions

The workflow requires the following permissions (already configured):
- `contents: write` - To checkout the repository
- `pull-requests: write` - For PR operations
- `issues: write` - For backward compatibility (may not be needed with direct Copilot API calls)

### Testing

You can test this workflow configuration by running:

```bash
bash .github/workflows/test-start-a-new-task.sh
```

This test script should validate:
- File existence (prompt file and agent-copilot binary)
- YAML syntax
- Correct workflow trigger configuration
- Required permissions

