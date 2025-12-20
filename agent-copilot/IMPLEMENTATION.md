# Agent Copilot - Implementation Notes

## Purpose
This project was created to automate the creation of GitHub Copilot agent tasks using the GitHub Copilot API directly. It replicates the functionality of the `gh agent-task create` command, using the same Copilot Jobs API at `api.githubcopilot.com`.

## Key Changes
The tool uses the GitHub Copilot Jobs API (`https://api.githubcopilot.com/agents/swe/v1/jobs/{owner}/{repo}`) to create agent tasks. This is the same API endpoint that `gh agent-task create` uses under the hood, providing direct integration with GitHub Copilot's agent system.

## Usage Example
```bash
agent-copilot \
  --repo jim-hill-r/monorepo \
  --title "Start a new task" \
  --prompt-file prompts/start-a-new-task.md \
  --token $GITHUB_TOKEN
```

## Integration with Workflows
This tool can be used in GitHub Actions workflows or locally to create agent tasks programmatically. It's particularly useful for:
- Automating agent task creation without creating issues
- Direct integration with GitHub Copilot's agent system
- CI/CD integration for automated agent task management

## Testing
The project includes unit tests for:
- File reading operations
- JSON serialization
- Error handling for missing files

Run tests with:
```bash
cargo test
```

## Building
Build the release binary with:
```bash
cargo build --release
```

The binary will be available at `target/release/agent-copilot`.

## Dependencies
All dependencies are standard, well-maintained Rust crates:
- `clap`: CLI argument parsing
- `reqwest`: HTTP client (blocking mode)
- `serde`/`serde_json`: JSON serialization
- `anyhow`: Error handling
- `tempfile`: Testing utilities (dev dependency)

## Security Considerations
- GitHub token is never logged or printed
- Uses HTTPS for all API communication
- Validates file existence before reading
- Proper error messages without exposing sensitive data
