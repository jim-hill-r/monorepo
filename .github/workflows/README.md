# GitHub Actions Workflows

## start-a-new-task.yml

This workflow automatically triggers a new GitHub Copilot agent task after a PR created by the Copilot agent is merged.

### Setup Requirements

The `gh agent-task create` command requires OAuth authentication, which the default `GITHUB_TOKEN` does not provide. To use this workflow, you need to:

1. **Create a Personal Access Token (PAT)**:
   - Go to GitHub Settings → Developer settings → Personal access tokens → Fine-grained tokens (or Classic tokens)
   - Create a new token with the following scopes:
     - `repo` (Full control of private repositories)
     - `workflow` (Update GitHub Action workflows)
   - Set an appropriate expiration date

2. **Add the PAT as a repository secret**:
   - Go to your repository → Settings → Secrets and variables → Actions
   - Create a new repository secret named `GH_PAT`
   - Paste your Personal Access Token as the value

3. **The workflow will automatically**:
   - Check if the `GH_PAT` secret is configured
   - Provide helpful error messages if the token is missing
   - Trigger a new agent task when a Copilot PR is merged

### Why is a PAT required?

The `gh agent-task` command is a preview feature that requires OAuth authentication. The standard `GITHUB_TOKEN` provided by GitHub Actions is an installation access token that doesn't have the OAuth scopes required by the `gh` CLI for agent task operations.

### Testing

You can test this workflow configuration by running:

```bash
bash .github/workflows/test-start-a-new-task.sh
```

This test script validates:
- File existence
- YAML syntax
- Correct authentication token usage (PAT instead of GITHUB_TOKEN)
- Proper error handling for missing tokens
- Workflow trigger configuration
