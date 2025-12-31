# Cast Toolchain Management Design

## Overview

Cast provides commands for managing development tools required by different project types. The `cast install` command installs required tools, while the `cast toolchain` command provides utilities for checking and listing installed tools. This design document outlines the command structure and implementation considerations.

## Motivation

Different project frameworks require different tooling beyond the Rust toolchain:

- **Dioxus projects**: Require Rust toolchain, Dioxus CLI (`dx`), Node.js/npm, and Playwright
- **Cloudflare Pages projects**: Require Rust toolchain, Wrangler CLI, Node.js/npm
- **Pure Rust projects**: Only require Rust toolchain (rustc, cargo, rustfmt, clippy)

Currently, developers and CI workflows must manually install these tools. The `cast install` and `cast toolchain` commands automate this process by reading the Cast configuration and managing the appropriate tools.

## Command Structure

### Install Command (Top-Level)

```bash
cast install [OPTIONS]
```

The `install` command is a top-level command for installing required development tools.

### Toolchain Subcommands

```bash
cast toolchain <SUBCOMMAND>
```

The `toolchain` command provides utilities for checking and listing tools, following the pattern of `rustup toolchain` and similar CLI tools.

## Commands

### 1. `install` - Install Required Tools

```bash
cast install [OPTIONS]
```

Installs all tools required by the current project based on its Cast configuration.

**Options:**

- `--tool <TOOL>` - Install a specific tool instead of all tools
  - Possible values: `node`, `npm`, `playwright`, `dx`, `wrangler`
  - Can be specified multiple times to install multiple specific tools
  - Example: `cast install --tool dx --tool playwright`

- `--skip <TOOL>` - Skip installation of a specific tool
  - Possible values: Same as `--tool`
  - Useful when some tools are already installed via system package managers
  - Example: `cast install --skip node`

- `--dry-run` - Show what would be installed without actually installing
  - Useful for CI/CD pipelines to understand requirements
  - Example: `cast install --dry-run`

- `--force` - Force reinstallation even if tools are already installed
  - Useful for upgrading or fixing corrupted installations
  - Example: `cast install --force`

**Behavior:**

1. Attempts to read Cast configuration from `Cargo.toml` or `Cast.toml` (if available)
2. Determines required tools based on:
   - `framework` field (if Cast configuration is found)
   - Default tools (rustc, cargo, rustfmt, clippy, git-lfs) if no configuration is found
3. Checks if tools are already installed
4. Installs missing tools using appropriate methods:
   - **Dioxus CLI**: `cargo install dioxus-cli --version X.Y.Z`
   - **Wrangler**: `cargo install wrangler` or `npm install -g wrangler`
   - **Playwright**: `npm ci` (if package.json exists) followed by `npx playwright install --with-deps chromium`
   - **Node.js/npm**: Provides guidance to install via system package manager (e.g., apt, brew, winget)
5. Outputs success/failure status for each tool

**Exit Codes:**

- `0` - All required tools installed successfully
- `1` - One or more tools failed to install

**Examples:**

```bash
# Install all required tools for the current project
cast install

# Install only the Dioxus CLI
cast install --tool dx

# Install all tools except Node.js (already installed via system)
cast install --skip node

# See what would be installed without installing
cast install --dry-run

# Force reinstall all tools
cast install --force
```

### 2. `toolchain check` - Verify Tools Are Installed

```bash
cast toolchain check [OPTIONS]
```

Verifies that all required tools are installed and outputs their versions.

**Options:**

- `--verbose` / `-v` - Show detailed version information for each tool
- `--json` - Output results in JSON format for programmatic use

**Behavior:**

1. Reads Cast configuration
2. Determines required tools
3. Checks if each tool is installed and gets version
4. Reports status and versions

**Exit Codes:**

- `0` - All required tools are installed
- `1` - One or more required tools are missing
- `2` - Cast configuration not found or invalid

**Examples:**

```bash
# Check if all required tools are installed
cast toolchain check

# Show detailed version information
cast toolchain check --verbose

# Output in JSON format for CI/CD
cast toolchain check --json
```

**Sample Output (text format):**

```
Checking toolchain for dioxus project...
✓ rustc 1.75.0 (installed)
✓ cargo 1.75.0 (installed)
✓ rustfmt 1.75.0 (installed)
✓ clippy 1.75.0 (installed)
✓ dx 0.7.2 (installed)
✓ node 20.10.0 (installed)
✓ npm 10.2.3 (installed)
✗ playwright (not installed)

Status: 1 tool missing
```

**Sample Output (JSON format):**

```json
{
  "framework": "dioxus",
  "tools": [
    {"name": "rustc", "required": true, "installed": true, "version": "1.75.0"},
    {"name": "cargo", "required": true, "installed": true, "version": "1.75.0"},
    {"name": "rustfmt", "required": true, "installed": true, "version": "1.75.0"},
    {"name": "clippy", "required": true, "installed": true, "version": "1.75.0"},
    {"name": "dx", "required": true, "installed": true, "version": "0.7.2"},
    {"name": "node", "required": true, "installed": true, "version": "20.10.0"},
    {"name": "npm", "required": true, "installed": true, "version": "10.2.3"},
    {"name": "playwright", "required": true, "installed": false, "version": null}
  ],
  "all_installed": false,
  "missing_count": 1
}
```

### 3. `toolchain list` - List Installed Tools

```bash
cast toolchain list [OPTIONS]
```

Lists all tools managed by Cast and their installation status.

**Options:**

- `--required-only` - Only show tools required by the current project
- `--all` - Show all tools that Cast can manage, regardless of project requirements
- `--json` - Output results in JSON format

**Behavior:**

1. If `--required-only`: Lists only tools required by current project
2. If `--all`: Lists all tools that Cast can manage
3. For each tool, shows: name, installed status, version (if installed)

**Exit Codes:**

- `0` - Success
- `2` - Cast configuration not found (when using `--required-only`)

**Examples:**

```bash
# List all tools managed by Cast
cast toolchain list --all

# List only tools required by current project
cast toolchain list --required-only

# List in JSON format
cast toolchain list --json
```

## Tool Detection Logic

The command determines required tools based on the Cast configuration:

### Framework-Based Requirements

| Framework | Required Tools |
|-----------|---------------|
| `dioxus` | rustc, cargo, rustfmt, clippy, git-lfs, dx, node, npm, playwright |
| `cloudflare-pages` | rustc, cargo, rustfmt, clippy, git-lfs, wrangler, node, npm |
| None (pure Rust) | rustc, cargo, rustfmt, clippy, git-lfs |

**Note**: Git LFS (git-lfs) is always required for all projects to handle large files in the repository.

### Additional Considerations

1. **Node.js detection**: If a `package.json` exists, Node.js and npm are required regardless of framework
2. **Playwright detection**: If a `playwright.config.ts` or `playwright.config.js` exists, Playwright is required. The installation automatically includes chromium browser and headless shell with system dependencies.
3. **Wrangler detection**: If a `wrangler.toml` exists, Wrangler is required

## Installation Methods

### Cargo-based Tools

Tools installed via `cargo install`:

- `dx` (Dioxus CLI): `cargo install dioxus-cli --version 0.7.2`
- `wrangler` (alternative to npm): `cargo install wrangler`

**Version management**: 
- Use specific versions when available (e.g., `dx` should be `0.7.2`)
- For other tools, install latest stable version

### NPM-based Tools

Tools installed via npm:

- `wrangler` (primary method): `npm install -g wrangler`
- `playwright`: `npm ci` (from package.json) then `npx playwright install --with-deps chromium`
  - **Important**: The toolchain check verifies that chromium browser is actually installed, not just the npm package
  - Uses `npx playwright install --list` to verify chromium installation

### System Package Manager Tools

Tools that should be installed via system package manager:

- `node` / `npm`: Provide guidance to user
  - Linux: `sudo apt install nodejs npm` or equivalent
  - macOS: `brew install node`
  - Windows: `winget install OpenJS.NodeJS`

- `git-lfs`: Provide guidance to user
  - Linux: `sudo apt install git-lfs && git lfs install`
  - macOS: `brew install git-lfs && git lfs install`
  - Windows: `winget install GitHub.GitLFS`

**Guidance approach**: Instead of attempting to install Node.js, the command should:
1. Detect if Node.js is already installed
2. If not, provide clear instructions for the user's platform
3. Exit with appropriate error code if Node.js is required but not installed

## Error Handling

The command should handle common error scenarios:

1. **No Cast configuration found**: Install default tools (Rust toolchain and git-lfs)
2. **Tool installation fails**: Show clear error with suggestions (e.g., network issues, permissions)
3. **Unsupported platform**: Detect platform and show appropriate error if tool isn't available
4. **Conflicting versions**: Warn if an incompatible version is installed

## Progress Output

During installation, the command should:

1. Show progress for each tool being installed
2. Use appropriate progress indicators (spinner, progress bar, or simple messages)
3. Show success/failure for each tool
4. Provide summary at the end

Example:

```
Installing toolchain for dioxus project...
✓ Rust toolchain already installed
⠋ Installing Dioxus CLI (dx)...
```

## CI/CD Integration

The command is designed for use in CI/CD pipelines:

1. **GitHub Actions integration**: 
   - Workflows should install Rust via `actions-rust-lang/setup-rust-toolchain`
   - Then use `cast install` for all other tools
   
2. **Fast execution**: 
   - Check before installing to avoid unnecessary work
   - Cache-friendly (works with GitHub Actions cache)

3. **Reproducible builds**: 
   - Use specific versions where critical (e.g., `dx 0.7.2`)
   - Document version requirements in project configuration

## Implementation Phases

Based on the root ISSUES.md, implementation will be broken into phases:

### Phase 1: Foundation (Issues 7-10)
- Add `Install` command variant to args.rs (now complete - moved to top-level)
- Create toolchain.rs module with basic structure (complete)
- Implement toolchain detection logic (complete)
- Add unit tests (complete)

### Phase 2: Installation (Issues 13-18)
- Implement Node.js detection and guidance (complete)
- Implement npm package installation (complete)
- Implement Dioxus CLI installation (complete)
- Implement Wrangler CLI installation (complete)
- Add version checking and upgrade logic (complete)
- Add comprehensive tests (complete)

### Phase 3: Platform Support (Issues 21-23)
- Test on Linux (GitHub Actions) (complete)
- Add macOS support if different (complete)
- Document Windows considerations (complete)

### Phase 4: Documentation (Issues 25-27)
- Update cast/README.md (complete)
- Create additional docs in cast/docs/ (complete)
- Update standards documents (complete)

### Phase 5: Integration (Issues 29-32)
- Update GitHub workflows to use `cast install`
- Update workflow conventions documentation
- Add copilot-instructions guidance
- Test in real PRs

### Phase 6: Enhancements (Issues 34-36)
- Implement `list` subcommand
- Implement `check` subcommand
- Add tests for all subcommands

## Future Enhancements

Potential future additions (not in initial scope):

1. **Custom tool versions**: Allow projects to specify tool versions in Cast.toml
2. **Tool aliases**: Support alternative installation methods
3. **Offline mode**: Support air-gapped environments with pre-downloaded tools
4. **Auto-update**: Automatically check for and update outdated tools
5. **Uninstall**: Remove tools that are no longer needed

## Compatibility

- **Minimum Rust version**: 1.70+ (aligned with Cast project requirements)
- **Supported platforms**: Linux, macOS, Windows (WSL)
- **Requires**: Rust toolchain must be installed before using this command

## Platform-Specific Considerations

### Windows Support

The `cast toolchain` command supports Windows environments, with the following considerations:

#### Recommended Environment: WSL2

**Windows Subsystem for Linux (WSL2)** is the recommended environment for running Cast projects on Windows:

- **Best compatibility**: WSL2 provides a full Linux environment, ensuring compatibility with all tools
- **Native Linux tools**: All installation commands work as documented for Linux
- **Performance**: Better file system performance for Rust builds compared to native Windows
- **Testing**: Primary Windows testing environment for GitHub Actions and CI/CD

**Setup WSL2:**
```bash
# Install WSL2 with Ubuntu (PowerShell as Administrator)
wsl --install

# Once in WSL2, install Rust and other dependencies
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Native Windows Support

Cast toolchain also supports native Windows environments with these considerations:

**Tool Installation Methods:**

1. **Node.js and npm**
   - **Recommended**: Use `winget` (Windows Package Manager)
     ```powershell
     winget install OpenJS.NodeJS
     ```
   - **Alternative**: Download installer from [nodejs.org](https://nodejs.org/)
   - **Note**: Requires restart or new terminal after installation to update PATH

2. **Cargo-based tools** (dx, wrangler)
   - Work identically to Linux/macOS
   - Installed via `cargo install` command
   - Example: `cargo install dioxus-cli --version 0.7.2`

3. **Playwright**
   - Installation via `npm ci` and `npx playwright install` works on native Windows
   - **Important**: May require administrator privileges for browser dependencies
   - **Browser installation path**: `%USERPROFILE%\AppData\Local\ms-playwright`
   - **Known limitation**: `--with-deps` flag for system dependencies only works on Linux; on Windows, browsers are downloaded without system dependencies

**Path Considerations:**

- Windows uses different path separators (`\` vs `/`)
- Cast commands handle path normalization automatically
- Environment variables use Windows format (e.g., `%USERPROFILE%` instead of `$HOME`)

**Permission Issues:**

Native Windows may require different permissions for certain operations:
- Some npm global installs may need administrator privileges
- Playwright browser installation may require administrator access
- Consider using `--skip node` if Node.js is already installed via system installer

**Known Limitations on Native Windows:**

1. **Shell differences**: PowerShell syntax differs from bash (commands in docs assume bash/WSL)
2. **Line endings**: Git may need configuration for CRLF vs LF line endings:
   ```bash
   git config --global core.autocrlf input
   ```
3. **Symlinks**: Some tools may have issues with symbolic links on Windows filesystems
4. **Case sensitivity**: Windows filesystem is case-insensitive by default (unlike Linux/macOS)

**Testing Recommendations:**

- **CI/CD**: Use WSL2-based environments for GitHub Actions (runs on Linux runners)
- **Local development**: WSL2 provides best compatibility and performance
- **Native Windows**: Supported but may encounter platform-specific issues with certain tools

**Troubleshooting:**

If encountering issues on native Windows:
1. Try running in WSL2 to verify it's a Windows-specific issue
2. Check that all tools are in PATH: `where node`, `where cargo`, `where dx`
3. Ensure PowerShell execution policy allows scripts: `Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser`
4. For npm permission errors, avoid using `sudo` on Windows; instead run as administrator or fix npm permissions

## References

- Similar commands: `rustup toolchain`, `npm install`, `cargo install`
- Cast configuration: cast/src/config.rs
- Framework requirements: See copilot-instructions.md
