# Agent Copilot

A Rust binary that calls GitHub Copilot to start an agent task directly using the GitHub API.

## Description

This tool automates the creation of GitHub Copilot agent tasks using the GitHub API, replacing the previous approach of creating GitHub issues. It reads agent prompt files and creates agent tasks programmatically, providing a more direct integration with GitHub Copilot's agent system.

## Building

To build the project:

```bash
cargo build
```

To build in release mode:

```bash
cargo build --release
```

## Running in Development Mode

You can run the CLI directly in development mode without building a release binary:

```bash
cargo run -- --repo jim-hill-r/monorepo --title "Start a new task" --prompt-file ../../.github/agent-prompts/start-a-new-task.md --token $GITHUB_TOKEN
```

This is useful during development and testing as it avoids the longer compile times of release builds.

## Usage

```bash
agent-copilot --repo <OWNER/REPO> --title <TITLE> --prompt-file <PATH> --token <GITHUB_TOKEN>
```

### Arguments

- `--repo`: Repository in the format `owner/repo` (e.g., `jim-hill-r/monorepo`)
- `--title`: Title for the agent task
- `--prompt-file`: Path to the agent prompt file
- `--token`: GitHub personal access token with appropriate permissions

### Example

```bash
agent-copilot \
  --repo jim-hill-r/monorepo \
  --title "Start a new task" \
  --prompt-file .github/agent-prompts/start-a-new-task.md \
  --token $GITHUB_TOKEN
```

## GitHub Token

### How to Obtain a GitHub Token

To use this tool, you need a GitHub Personal Access Token with the appropriate permissions:

1. Go to GitHub Settings: Click your profile photo → **Settings**
2. Navigate to **Developer settings** (at the bottom of the left sidebar)
3. Click **Personal access tokens** → **Tokens (classic)**
4. Click **Generate new token** → **Generate new token (classic)**
5. Give your token a descriptive name (e.g., "agent-copilot CLI")
6. Set an expiration date (or choose "No expiration" for tokens you'll use long-term)
7. Select the following scopes:
   - `repo` - Full control of private repositories (includes `repo:status`, `repo_deployment`, `public_repo`, `repo:invite`, `security_events`)
   - Optionally, `workflow` if you need to trigger workflows
8. Click **Generate token**
9. **Important**: Copy the token immediately - you won't be able to see it again!

Store the token securely. Never commit it to source code or share it publicly.

### Using the Token

You can provide the token in two ways:

**Option 1: Environment Variable (Recommended)**

```bash
export GITHUB_TOKEN=your_token_here
agent-copilot --repo jim-hill-r/monorepo --title "Start a new task" --prompt-file .github/agent-prompts/start-a-new-task.md
```

**Option 2: Command-line Argument**

```bash
agent-copilot \
  --repo jim-hill-r/monorepo \
  --title "Start a new task" \
  --prompt-file .github/agent-prompts/start-a-new-task.md \
  --token ghp_your_token_here
```

## Features

- Creates GitHub Copilot agent tasks directly via the GitHub API
- Reads agent prompts from markdown files
- Supports GitHub authentication via token
- Direct integration with GitHub Copilot's agent system

## Important Notes

This tool uses the GitHub Copilot Tasks API endpoint (`/repos/{owner}/{repo}/copilot/tasks`). The exact API response structure may vary based on GitHub's implementation. If you encounter issues with the API response format, please check the error messages and adjust the response structure in `src/main.rs` accordingly.

## Dependencies

- `clap`: Command-line argument parsing
- `reqwest`: HTTP client for GitHub API calls
- `serde`: Serialization/deserialization
- `anyhow`: Error handling
