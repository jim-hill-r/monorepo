# Agent Copilot

A Rust binary that calls GitHub Copilot to start an agent task similar to the workflow `start-a-new-task.yml`.

## Description

This tool automates the creation of GitHub issues that trigger GitHub Copilot agents, mimicking the behavior of the `start-a-new-task.yml` workflow. It reads agent prompt files and creates issues programmatically using the GitHub API.

## Building

To build the project:

```bash
cargo build
```

To build in release mode:

```bash
cargo build --release
```

## Usage

```bash
agent-copilot --repo <OWNER/REPO> --title <TITLE> --prompt-file <PATH> --token <GITHUB_TOKEN>
```

### Arguments

- `--repo`: Repository in the format `owner/repo` (e.g., `jim-hill-r/monorepo`)
- `--title`: Title for the GitHub issue
- `--prompt-file`: Path to the agent prompt file
- `--token`: GitHub personal access token with `repo` scope

### Example

```bash
agent-copilot \
  --repo jim-hill-r/monorepo \
  --title "Start a new task" \
  --prompt-file .github/agent-prompts/start-a-new-task.md \
  --token $GITHUB_TOKEN
```

## Environment Variables

Instead of passing the token as a CLI argument, you can set it as an environment variable:

```bash
export GITHUB_TOKEN=your_token_here
agent-copilot --repo jim-hill-r/monorepo --title "Start a new task" --prompt-file .github/agent-prompts/start-a-new-task.md
```

## Features

- Creates GitHub issues programmatically via the GitHub API
- Reads issue body from markdown files
- Supports GitHub authentication via token
- Mimics the behavior of GitHub Actions workflows

## Dependencies

- `clap`: Command-line argument parsing
- `reqwest`: HTTP client for GitHub API calls
- `tokio`: Async runtime
- `serde`: Serialization/deserialization
- `anyhow`: Error handling
