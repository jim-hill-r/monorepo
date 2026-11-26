# GitHub Actions Workflows

## cast-ci.yml

This workflow automatically runs `cast ci` for any project that has changes in a pull request.

### How It Works

1. **Trigger**: The workflow runs on pull request events (opened, synchronized, reopened).

2. **Build and Run**: 
   - Sets up the Rust and Node.js toolchains
   - Builds the `cast` CLI from `cast/cli`
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
4. The `cast/cli` project must be buildable

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
   - Builds the `cast` CLI from `cast/cli`
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
4. The `cast/cli` project must be buildable

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
   - Builds the `cast` CLI from `cast/cli`
   - Runs `cast cd` for each project with new artifacts

4. **Results**: 
   - Groups output by project for easy reading
   - Fails the workflow if any project's CD fails

### Setup Requirements

The workflow requires:
1. Rust toolchain (automatically installed by the workflow)
2. Node.js toolchain (automatically installed by the workflow)
3. Projects must have a `Cast.toml` file in their root directory
4. The `cast/cli` project must be buildable

### Permissions

The workflow requires the following permissions:
- `contents: read` - To checkout the repository and read files

### Integration with Trunk CI

This workflow is designed to run after the Trunk CI workflow, which:
1. Runs `cast ci --release` on changed projects
2. Builds Linux artifacts in `artifacts/x86_64-unknown-linux-gnu/` directories
3. Commits these artifacts back to the main branch

When Trunk CI commits new artifacts, this CD workflow automatically triggers to deploy those projects.

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


## all-delete-branches.yml

Deletes remote branches without needing a local clone. Run it manually from the Actions tab (or the `workflow_dispatch` API) with:

- `branches`: branch names separated by spaces or newlines.
- `dry_run`: defaults to `true`, which only reports what would be deleted. Set it to `false` to delete.

The workflow never deletes the default branch and skips any branch that is the head of an open pull request. Branches that no longer exist are reported as not found. The run summary lists how many branches were deleted, skipped, missing, or failed.

### Permissions

- `contents: write` - To delete branch refs
- `pull-requests: read` - To find branches with open pull requests

## pull-request-agent-skills.yml

Validates the agent skills in `.agents/skills/` against the
[Agent Skills specification](https://agentskills.io/specification) on pull
requests that touch `.agents/`, `.claude/`, `AGENTS.md` or `CLAUDE.md`. It uses
the pinned `skills-ref` reference validator and also checks that
`.claude/skills` is still a symlink to `../.agents/skills`. Can also be run
manually via `workflow_dispatch`.

Tests: `monorepo/workflow_tests/tests/agent_skills_workflow_tests.rs`.
