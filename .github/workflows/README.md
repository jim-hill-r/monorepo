# GitHub Actions Workflows

## cast-ci.yml

This workflow automatically runs `cast ci` for any project that has changes in a pull request.

### How It Works

1. **Trigger**: The workflow runs on pull request events (opened, synchronized, reopened).

2. **Build and Run**: 
   - Sets up the Rust and Node.js toolchains
   - Builds the `cast` CLI from `cast/cast_cli`
   - Runs `cast ci --only-changed --recursive 2` from the repository root
   - This recursively finds all projects with `Cast.toml` up to 2 levels deep and runs CI checks only on projects with changes

3. **Changed Project Detection**: 
   - The `--only-changed` flag makes `cast ci` check each project for changes compared to the origin's default branch
   - Projects with no changes are automatically skipped
   - The `--recursive 2` flag ensures all Cast projects up to 2 levels deep are checked

4. **Results**: 
   - `cast ci` automatically installs required tools for each project
   - Runs all CI checks (fmt, clippy, build, test) for changed projects
   - Fails the workflow if any project's CI check fails

### Setup Requirements

The workflow requires:
1. Rust toolchain (automatically installed by the workflow)
2. Node.js toolchain (automatically installed by the workflow)
3. Projects must have a `Cast.toml` file in their root directory
4. The `cast/cast_cli` project must be buildable

### Permissions

The workflow requires the following permissions:
- `contents: read` - To checkout the repository and read files
- `pull-requests: read` - To access PR information

## nightly.yml

This workflow runs `cast ci` recursively across all projects in the monorepo on a nightly schedule.

### How It Works

1. **Trigger**: The workflow runs automatically at 2 AM Pacific Time (10 AM UTC) every day, and can also be triggered manually via `workflow_dispatch`.

2. **Build and Run**: 
   - Sets up the Rust toolchain
   - Builds the `cast` CLI from `cast/cast_cli`
   - Runs `cast ci --recursive 2 --check` from the repository root
   - This recursively finds all projects with `Cast.toml` up to 2 levels deep and runs CI checks on them

3. **Results**: 
   - Fails the workflow if any project's CI check fails
   - Provides a comprehensive health check of the entire monorepo

### Setup Requirements

The workflow requires:
1. Rust toolchain (automatically installed by the workflow)
2. Node.js toolchain (automatically installed by the workflow)
3. Projects must have a `Cast.toml` file in their root directory
4. The `cast/cast_cli` project must be buildable

### Permissions

The workflow requires the following permissions:
- `contents: read` - To checkout the repository and read files

## cast-cd.yml (CD)

This workflow automatically runs `cast cd` for any project when new Linux build artifacts are committed to the main branch.

### How It Works

1. **Trigger**: The workflow runs when artifacts are pushed to the main branch. Specifically, it triggers on changes to `**/artifacts/x86_64-unknown-linux-gnu/**` paths.

2. **Artifact Detection**: 
   - Gets the list of files changed in the latest commit
   - Filters for files in `artifacts/x86_64-unknown-linux-gnu/` directories
   - Extracts the parent project directory for each artifact
   - Collects unique project directories that have new Linux artifacts

3. **Build and Run**: 
   - Sets up the Rust and Node.js toolchains
   - Builds the `cast` CLI from `cast/cast_cli`
   - Runs `cast cd` for each project with new artifacts

4. **Results**: 
   - Groups output by project for easy reading
   - Fails the workflow if any project's CD fails

### Setup Requirements

The workflow requires:
1. Rust toolchain (automatically installed by the workflow)
2. Node.js toolchain (automatically installed by the workflow)
3. Projects must have a `Cast.toml` file in their root directory
4. The `cast/cast_cli` project must be buildable

### Permissions

The workflow requires the following permissions:
- `contents: read` - To checkout the repository and read files

### Integration with Trunk CI

This workflow is designed to run after the Trunk CI workflow, which:
1. Runs `cast ci --release` on changed projects
2. Builds Linux artifacts in `artifacts/x86_64-unknown-linux-gnu/` directories
3. Commits these artifacts back to the main branch

When Trunk CI commits new artifacts, this CD workflow automatically triggers to deploy those projects.

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
cd monorepo/workflow_tests
cargo test start_a_new_task_workflow_tests
```

This Rust test suite validates:
- File existence (prompt file and agent-copilot binary)
- YAML syntax
- Correct workflow trigger configuration
- Required permissions
- Concurrency control logic

## codeql.yml

This workflow runs CodeQL security scanning on the codebase, analyzing JavaScript/TypeScript and GitHub Actions workflows but **not Rust**.

### Why Rust is Excluded

- **Performance**: Rust CodeQL analysis takes ~26 minutes per scan
- **Scope**: The monorepo contains ~20+ Rust projects
- **Efficiency**: Most PRs don't modify all Rust code
- **Alternative**: Rust security is checked by `clippy` in `cast ci`

### How It Works

1. **Trigger**: The workflow runs on:
   - Push to `main` branch
   - Pull requests to `main` branch
   - Weekly schedule (Sundays at 2 AM UTC)

2. **Language Scanning**: 
   - **JavaScript/TypeScript**: ~1 minute scan time
   - **GitHub Actions**: ~40 seconds scan time
   - **Rust**: Excluded (would take ~26 minutes)

3. **Analysis**:
   - Uses GitHub's CodeQL action
   - Automatically builds the code (autobuild)
   - Uploads results to GitHub Security tab

### Relationship to GitHub's Default CodeQL

This custom workflow **replaces** GitHub's default CodeQL setup. When a custom `.github/workflows/codeql.yml` exists:
- GitHub's automatic default CodeQL is disabled
- This custom workflow takes over
- You have full control over which languages are scanned

### Re-enabling Rust Scanning

If Rust security scanning is needed:

**Option 1: Enable in this workflow**
- Edit `.github/workflows/codeql.yml`
- Add `'rust'` to the `matrix.language` array
- Note: This will add ~26 minutes to each PR/push

**Option 2: Create a separate scheduled workflow**
- Create a new workflow that only runs weekly
- Only scans Rust
- Doesn't block PRs with long scan times

### Permissions

The workflow requires:
- `contents: read` - To checkout the repository
- `security-events: write` - To upload CodeQL results
- `actions: read` - To read workflow information

