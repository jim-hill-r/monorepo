# Agent Copilot

A Rust binary that calls GitHub Copilot to start an agent task directly using the GitHub API.

## Description

This tool automates the creation of GitHub Copilot agent tasks using the GitHub Copilot API. It reads agent prompt files and creates jobs programmatically using the same API endpoint that the `gh agent-task create` command uses.

## Building

To build the project:

```bash
cargo build
```

To build in release mode:

```bash
cargo build --release
```

## Cross-Compiling for Linux x86_64

If you need to compile the project for Linux x86_64 (compatible with Ubuntu latest) from another platform, you can use the `cross` tool. This is particularly useful when building on macOS or Windows for deployment on Linux servers.

### Installing Cross

First, install the `cross` tool:

```bash
cargo install cross --git https://github.com/cross-rs/cross
```

Note: `cross` requires Docker to be installed and running on your system, as it uses Docker containers to provide a consistent cross-compilation environment.

### Cross-Compiling to Linux x86_64

To compile for Linux x86_64 (Ubuntu compatible):

```bash
cross build --target x86_64-unknown-linux-gnu --release
```

The compiled binary will be located at:

```
target/x86_64-unknown-linux-gnu/release/agent-copilot
```

### Available Linux Targets

The `cross` tool supports various Linux targets:

- `x86_64-unknown-linux-gnu` - 64-bit Linux with GNU libc (Ubuntu, Debian, etc.)
- `x86_64-unknown-linux-musl` - 64-bit Linux with MUSL libc (static linking, smaller binaries)
- `aarch64-unknown-linux-gnu` - 64-bit ARM Linux (for ARM servers)

Example for MUSL (fully static binary):

```bash
cross build --target x86_64-unknown-linux-musl --release
```

### Verifying the Binary

After cross-compilation, you can verify the binary was built for the correct platform:

```bash
file target/x86_64-unknown-linux-gnu/release/agent-copilot
```

Expected output:
```
target/x86_64-unknown-linux-gnu/release/agent-copilot: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2, for GNU/Linux 3.2.0, with debug_info, not stripped
```

### Troubleshooting Cross-Compilation

If you encounter issues:

1. **Docker not running**: Ensure Docker is installed and running
   ```bash
   docker --version
   docker ps
   ```

2. **Permission issues**: You may need to run Docker without sudo:
   ```bash
   sudo usermod -aG docker $USER
   # Log out and back in for this to take effect
   ```

3. **Network issues**: The first run downloads Docker images, which may take time depending on your connection

4. **OpenSSL dependency issues**: When cross-compiling, you may encounter errors like "Could not find directory of OpenSSL installation". This happens because the project depends on `reqwest` which uses OpenSSL by default. There are several solutions:

   **Option A: Use vendored OpenSSL (recommended for cross-compilation)**
   
   Add this feature to compile OpenSSL from source during the build:
   ```bash
   cross build --target x86_64-unknown-linux-gnu --release --features vendored-openssl
   ```
   
   You would need to add a `vendored-openssl` feature to `Cargo.toml`:
   ```toml
   [features]
   vendored-openssl = ["reqwest/native-tls-vendored"]
   ```
   
   **Option B: Use rustls instead of OpenSSL**
   
   Modify `Cargo.toml` to use rustls instead of native-tls:
   ```toml
   reqwest = { version = "0.11", features = ["json", "blocking", "rustls-tls"], default-features = false }
   ```
   
   **Option C: Install OpenSSL in the cross container**
   
   Create a `Cross.toml` file in the project root with:
   ```toml
   [target.x86_64-unknown-linux-gnu]
   pre-build = [
       "apt-get update",
       "apt-get install -y libssl-dev pkg-config",
   ]
   ```
   
   Note: The project currently uses the default OpenSSL backend, so you may need to apply one of these solutions for successful cross-compilation.

## Running in Development Mode

You can run the CLI directly in development mode without building a release binary:

```bash
cargo run -- --repo jim-hill-r/monorepo --title "Start a new task" --prompt-file ../../.github/agent-prompts/start-a-new-task.md --token $GITHUB_TOKEN
```

Note: The `--title` parameter is kept for backwards compatibility but is not used by the Copilot API. The problem statement from the prompt file is what matters.

This is useful during development and testing as it avoids the longer compile times of release builds.

## Usage

```bash
agent-copilot --repo <OWNER/REPO> --title <TITLE> --prompt-file <PATH> --token <GITHUB_TOKEN> [--base-branch <BRANCH>] [--custom-agent <AGENT>]
```

### Arguments

- `--repo`: Repository in the format `owner/repo` (e.g., `jim-hill-r/monorepo`)
- `--title`: Title for the agent task (kept for backwards compatibility, not used by API - may be removed in future versions)
- `--prompt-file`: Path to the agent prompt file containing the problem statement
- `--token`: GitHub personal access token with appropriate permissions
- `--base-branch`: (Optional) Base branch for the pull request
- `--custom-agent`: (Optional) Custom agent to use (e.g., 'my-agent' for '.github/agents/my-agent.md')

### Example

```bash
agent-copilot \
  --repo jim-hill-r/monorepo \
  --title "Start a new task" \
  --prompt-file .github/agent-prompts/start-a-new-task.md \
  --token $GITHUB_TOKEN

# With optional parameters
agent-copilot \
  --repo jim-hill-r/monorepo \
  --title "Start a new task" \
  --prompt-file .github/agent-prompts/start-a-new-task.md \
  --token $GITHUB_TOKEN \
  --base-branch main \
  --custom-agent my-custom-agent
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
   - `copilot` - Access to GitHub Copilot (required for creating agent tasks)
   - `repo` - Full control of private repositories (may also be required depending on your repository visibility)
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

- Creates GitHub Copilot agent tasks using the Copilot API (same as `gh agent-task create`)
- Reads agent prompts from markdown files
- Supports GitHub authentication via token
- Supports optional base branch and custom agent configuration
- Direct integration with the GitHub Copilot Jobs API at `api.githubcopilot.com`

## Important Notes

This tool uses the GitHub Copilot Jobs API endpoint (`https://api.githubcopilot.com/agents/swe/v1/jobs/{owner}/{repo}`), which is the same endpoint used by the `gh agent-task create` command. The token must have the `copilot` scope to create agent tasks. If you encounter a 401 or 403 error, ensure your token has the correct permissions as described in the "GitHub Token" section.

## Dependencies

- `clap`: Command-line argument parsing
- `reqwest`: HTTP client for GitHub API calls
- `serde`: Serialization/deserialization
- `anyhow`: Error handling
