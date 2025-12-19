# Agent Copilot - Implementation Notes

## Purpose
This project was created to automate the creation of GitHub issues that trigger GitHub Copilot agents, mimicking the behavior of `.github/workflows/start-a-new-task.yml`.

## Usage Example
```bash
agent-copilot \
  --repo jim-hill-r/monorepo \
  --title "Start a new task" \
  --prompt-file .github/agent-prompts/start-a-new-task.md \
  --token $GITHUB_TOKEN
```

## Integration with Workflows
This tool can be used in GitHub Actions workflows or locally to create issues programmatically. It's particularly useful for:
- Automating agent task creation
- Batch creation of issues
- CI/CD integration for automated issue management

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
