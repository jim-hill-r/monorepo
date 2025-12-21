# GitHub Actions Workflows

## cast-ci.yml

This workflow automatically runs `cast ci` for any project that has changes in a pull request.

### How It Works

1. **Trigger**: The workflow runs on pull request events (opened, synchronized, reopened).

2. **Changed Project Detection**: 
   - Gets the list of changed files between the base and head branches
   - For each changed file, walks up the directory tree to find a `Cast.toml` file
   - Collects unique project directories that have a `Cast.toml`

3. **Build and Run**: 
   - Sets up the Rust toolchain
   - Builds the `cast` CLI from `cast_cli`
   - Runs `cast ci` for each detected project

4. **Results**: 
   - Groups output by project for easy reading
   - Fails the workflow if any project's CI check fails

### Setup Requirements

The workflow requires:
1. Rust toolchain (automatically installed by the workflow)
2. Projects must have a `Cast.toml` file in their root directory
3. The `cast_cli` project must be buildable

### Permissions

The workflow requires the following permissions:
- `contents: read` - To checkout the repository and read files
- `pull-requests: read` - To access PR information

### Testing

You can test this workflow configuration by running:

```bash
cd workflow_tests
cargo test cast_ci_workflow_tests
```

These Rust tests validate:
- Workflow file existence and YAML syntax
- Correct trigger configuration
- Use of git diff for change detection
- Cast.toml detection logic
- Rust toolchain setup
- Cast CLI build and execution
- Error handling for git operations
- Proper fetching of commit SHAs
- Proper quoting of GitHub Actions expressions

### Error Handling

The workflow includes robust error handling for git operations:
- Explicitly fetches both base and head commits to ensure they're available
- Captures stderr from git diff to provide clear error messages
- Checks git diff exit code and fails gracefully with diagnostic output
- Uses `|| true` on fetch operations to prevent unnecessary failures

This ensures that the workflow fails fast with clear error messages when commits are unavailable, rather than continuing with corrupted state.

## cast-cd.yml

This workflow automatically runs `cast cd` for any project that has changes when a pull request is merged.

### How It Works

1. **Trigger**: The workflow runs when a pull request is closed and merged.

2. **Changed Project Detection**: 
   - Gets the list of changed files between the base and head branches
   - For each changed file, walks up the directory tree to find a `Cast.toml` file
   - Collects unique project directories that have a `Cast.toml`

3. **Build and Run**: 
   - Sets up the Rust toolchain
   - Builds the `cast` CLI from `cast_cli`
   - Runs `cast cd` for each detected project

4. **Results**: 
   - Groups output by project for easy reading
   - Fails the workflow if any project's CD fails

### Setup Requirements

The workflow requires:
1. Rust toolchain (automatically installed by the workflow)
2. Projects must have a `Cast.toml` file in their root directory
3. The `cast_cli` project must be buildable

### Permissions

The workflow requires the following permissions:
- `contents: read` - To checkout the repository and read files
- `pull-requests: read` - To access PR information

## start-a-new-task.yml

This workflow automatically creates a GitHub Copilot agent task after a PR created by the Copilot agent is merged.

### How It Works

1. **Trigger**: The workflow runs when a pull request is closed and merged, but only if it was created by the GitHub Copilot agent (`user.login == 'Copilot'`).

2. **Concurrency Check**: Before creating a new agent task, the workflow checks if there are any open PRs created by Copilot. If any active agent tasks exist, the workflow skips creating a new task to prevent running multiple agents concurrently.

3. **Agent Task Creation**: If no active agent tasks are found, the workflow uses the `agent-copilot` binary to create a new GitHub Copilot agent task with:
   - Title: "Start a new task"
   - Problem Statement: Content from `agent-copilot/prompts/start-a-new-task.md`
   - Repository: The current repository
   - Note: This directly creates an agent task using the GitHub Copilot API, bypassing the need to create an issue first.

4. **Authentication**: Uses the `START_NEW_AI_AGENT_TASK_WORKFLOW_PAT` secret for creating agent tasks, and the standard `GITHUB_TOKEN` for checking open PRs.

### Setup Requirements

The workflow requires:
1. The `agent-copilot` binary must be present at `agent-copilot/artifacts/x86_64-unknown-linux-gnu/agent-copilot`
2. The `START_NEW_AI_AGENT_TASK_WORKFLOW_PAT` secret with appropriate permissions for creating agent tasks
3. The `GITHUB_TOKEN` provided by GitHub Actions (automatically available)

### Permissions

The workflow requires the following permissions (already configured):
- `contents: write` - To checkout the repository
- `pull-requests: write` - For PR operations and checking open PRs
- `issues: write` - For backward compatibility (may not be needed with direct Copilot API calls)

### Concurrency Control

The workflow prevents multiple agent tasks from running concurrently by:
- Checking for open PRs created by the Copilot user before starting a new task
- Skipping task creation if any active agent tasks are found
- Logging the number of active agent tasks when skipping

This ensures that only one agent task runs at a time, preventing conflicts and resource contention.

### Testing

You can test this workflow configuration by running:

```bash
cd workflow_tests
cargo test start_a_new_task_workflow_tests
```

This Rust test suite validates:
- File existence (prompt file and agent-copilot binary)
- YAML syntax
- Correct workflow trigger configuration
- Required permissions
- Concurrency control logic

