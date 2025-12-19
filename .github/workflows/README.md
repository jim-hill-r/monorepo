# GitHub Actions Workflows

## start-a-new-task.yml

This workflow automatically creates a GitHub Issue that triggers GitHub Copilot agent after a PR created by the Copilot agent is merged.

### How It Works

1. **Trigger**: The workflow runs when a pull request is closed and merged, but only if it was created by the GitHub Copilot agent (`user.login == 'Copilot'`).

2. **Issue Creation**: The workflow creates a new GitHub Issue with:
   - Title: "Start a new task"
   - Body: Content from `.github/agent-prompts/start-a-new-task.md`
   - Note: The issue is not explicitly assigned to anyone. GitHub Copilot will automatically respond to issues created this way.

3. **Authentication**: Uses the standard `GITHUB_TOKEN` provided by GitHub Actions, which has sufficient permissions for creating issues.

### Setup Requirements

No additional setup is required! The workflow uses the default `GITHUB_TOKEN` which has the necessary permissions to create issues and assign them to @copilot.

### Permissions

The workflow requires the following permissions (already configured):
- `contents: write` - To checkout the repository
- `pull-requests: write` - For PR operations
- `issues: write` - To create issues

### Testing

You can test this workflow configuration by running:

```bash
bash .github/workflows/test-start-a-new-task.sh
```

This test script validates:
- File existence
- YAML syntax
- Uses `gh issue create` command
- Does not use `--assignee` flag (which can cause errors)
- Has required permissions
- Correct workflow trigger configuration

