# Cast

Highly opinionated tooling for rust monorepos.

This project aims to make managing Rust monorepos simple via a simple CLI that can be run locally, on CI, and in cloud environments.

A cast is a [group of crabs](https://www.originaldiving.com/blog/our-favourite-collective-nouns-for-sea-creatures#:~:text=A%20group%20of%20crabs%20is,crabs%20dominating%20access%20to%20food.).

# Dependencies

- Rust
- Cargo

# Build

- Run `cargo build --release`

# Features

## Development

### Running Server

Cast provides a `run` command that runs the appropriate server for a project.

```bash
cast run
```

This command automatically detects the project framework from the Cast configuration and runs:
- `dx serve` for Dioxus projects (when `framework = "dioxus"` is set in Cast.toml or Cargo.toml)
- `cargo run` for all other projects (default behavior)

The framework is determined by checking the `framework` field in the project's Cast configuration. Cast will check `Cargo.toml` for a `[package.metadata.cast]` section first, then fall back to `Cast.toml`.

Example usage in library code:

```rust
use cast_core::run;

// Run server on a project
run::run("/path/to/project").unwrap();
```

### Serving Static Files

Cast provides a `serve` command that serves static files from the current directory on a simple HTTP server.

```bash
cast serve
```

This command:
- Starts an HTTP server on `http://127.0.0.1:8000`
- Serves files from the current directory
- Automatically serves `index.html` when accessing directories
- Includes proper Content-Type headers for common file types (HTML, CSS, JS, images, etc.)
- Prevents directory traversal attacks

This is useful for:
- Testing static site builds locally
- Serving documentation
- Quick file sharing in development environments
- Testing Dioxus SSG (Static Site Generation) builds

Example usage in library code:

```rust
use cast_core::serve;

// Serve static files from a directory
serve::run("/path/to/static/files").unwrap();
```

## Toolchain Management

Cast provides toolchain management commands to help install and manage framework-specific development tools required by your projects.

### Overview

Different frameworks require different tooling beyond Rust:
- **Dioxus projects**: Require Dioxus CLI (`dx`), Node.js, npm, and Playwright for testing
- **Cloudflare Pages projects**: Require Wrangler CLI for deployment
- **Pure Rust projects**: Only require the Rust toolchain (rustc, cargo, rustfmt, clippy)

The `cast install` and `cast toolchain` commands help automate the installation and verification of these tools.

### Install Toolchain Dependencies

```bash
cast install
```

Installs all required tools for your project based on its framework configuration. If no Cast configuration is found, installs default tools (rustc, cargo, rustfmt, clippy, git-lfs).

**Options**:
- `--tool <TOOL>` - Install only a specific tool (e.g., nodejs, npm, playwright, dx, wrangler)
- `--skip <SKIP>` - Skip specific tools during installation (comma-separated list)
- `--dry-run` - Show what would be installed without actually installing
- `--force` - Force reinstall even if tools are already installed

**Examples**:
```bash
# Install all required tools for the current project
cast install

# See what would be installed without installing
cast install --dry-run

# Install only the Dioxus CLI
cast install --tool dx

# Install all tools except Node.js (already installed via system)
cast install --skip node

# Force reinstall all tools
cast install --force
```

**Notes**:
- Node.js and npm must be installed via your system package manager (apt, brew, winget)
- The command will provide installation instructions if Node.js/npm are not found
- Dioxus CLI is installed via `cargo install dioxus-cli --version 0.7.2`
- Playwright is installed via `npm ci` (if package.json exists) and `npx playwright install --with-deps chromium`
- Wrangler is installed via `npm install -g wrangler`

### Check Toolchain Status

```bash
cast toolchain check
```

Verifies that all required tools are installed and displays their versions.

**Options**:
- `--verbose` / `-v` - Show detailed version information for each tool
- `--json` - Output results in JSON format for programmatic use

**Examples**:
```bash
# Check if all required tools are installed
cast toolchain check

# Show detailed version information
cast toolchain check --verbose

# Output in JSON format for CI/CD
cast toolchain check --json
```

**Exit Codes**:
- `0` - All required tools are installed
- `1` - One or more required tools are missing

**Sample Output (text format)**:
```
Checking toolchain for dioxus project...
✓ cargo
✓ clippy
✗ dx (not installed)
✓ node
✓ npm
✗ playwright (not installed)
✓ rustc
✓ rustfmt

Status: 2 tools missing
```

**Sample Output (verbose)**:
```
Checking toolchain for dioxus project...
✓ cargo (1.75.0)
✓ clippy (0.1.75)
✗ dx (not installed)
✓ node (20.10.0)
✓ npm (10.2.3)
✗ playwright (not installed)
✓ rustc (1.75.0)
✓ rustfmt (1.6.0-stable)

Status: 2 tools missing
```

**Sample Output (JSON format)**:
```json
{
  "all_installed": false,
  "framework": "dioxus",
  "missing_count": 2,
  "tools": [
    {"name": "cargo", "required": true, "installed": true, "version": "1.75.0"},
    {"name": "clippy", "required": true, "installed": true, "version": "0.1.75"},
    {"name": "dx", "required": true, "installed": false, "version": null},
    {"name": "node", "required": true, "installed": true, "version": "20.10.0"},
    {"name": "npm", "required": true, "installed": true, "version": "10.2.3"},
    {"name": "playwright", "required": true, "installed": false, "version": null},
    {"name": "rustc", "required": true, "installed": true, "version": "1.75.0"},
    {"name": "rustfmt", "required": true, "installed": true, "version": "1.6.0-stable"}
  ]
}
```

### List Available Tools

List all tools and their installation status:

```bash
cast toolchain list
```

**Options**:
- `--required-only` - Show only tools required for the current project
- `--all` - Show all known tools, not just installed ones
- `--json` - Output results in JSON format

Example output:
```
rustc: 1.75.0 (installed)
cargo: 1.75.0 (installed)
rustfmt: 1.7.0-stable (installed)
clippy: 0.1.75 (installed)
dx: 0.7.2 (installed)
node: 20.10.0 (installed)
npm: 10.2.4 (installed)
playwright: not installed
```

With JSON output:
```bash
cast toolchain list --json
```

Example JSON output:
```json
{
  "tools": [
    {
      "name": "rustc",
      "installed": true,
      "version": "1.75.0"
    },
    {
      "name": "playwright",
      "installed": false,
      "version": null
    }
  ]
}
```

## CI/CD


### Building Projects

Cast provides a `build` command that builds Rust projects.

```bash
cast build
```

This will run `cargo build` in the current project directory. The command is designed to be consistent with other Cast commands and can be extended in the future with additional build functionality.

Example usage in library code:

```rust
use cast_core::build;

// Run build on a project
build::run("/path/to/project").unwrap();
```

### Running Tests

Cast provides a `test` command that runs tests for projects.

```bash
cast test
```

This command detects the project type and runs appropriate tests:
- For **Rust projects** (with Cargo.toml): Runs `cargo test`
- For **TypeScript/Node.js projects** (with package.json and test script): Runs `npm test`
- Projects can have both (e.g., Dioxus web apps with Playwright tests), and Cast will run both test suites

Example usage in library code:

```rust
use cast_core::test;

// Run tests on a project
test::run("/path/to/project").unwrap();
```

### Running CI Checks

Cast provides a `ci` command that runs standard project checks. This is designed to be used in CI workflows.

```bash
cast ci
```

For **Rust projects** (with Cargo.toml), this will run:
1. `cargo fmt --check` - Verify code formatting
2. `cargo clippy -- -D warnings` - Lint code for common mistakes
3. `cast build` - Ensure the project compiles (via `cargo build`)
4. `cast test` - Run all tests (via `cargo test`)
5. `cast publish` - Build release artifacts (binaries or web bundles)
6. Commit artifacts to git using git-lfs (if in a git repository)

For **TypeScript/Node.js projects** (with package.json), this will run:
1. `npm ci` - Install dependencies from lockfile (fast, reproducible)
2. `npm run lint` - Run linting (if script exists)
3. `npm run compile` - Compile TypeScript (if script exists)
4. `npm test` - Run tests (if script exists, e.g., Playwright tests)

Projects can have both Cargo.toml and package.json (e.g., Dioxus web apps with Playwright tests), and Cast will run both Rust and TypeScript CI checks.

**Artifact Committing**: After successful CI checks and publish, artifacts are automatically committed to git with git-lfs. This requires:
- Being in a git repository
- Having git-lfs installed
- Having changes in the `artifacts/` directory
- The repository's `.gitattributes` should already configure git-lfs for artifact files (e.g., `*.zip filter=lfs diff=lfs merge=lfs -text`)

If any check fails, the command will exit with an error. This makes it easy to integrate with CI systems like GitHub Actions.

Example usage in library code:

```rust
use cast_core::ci;

// Run CI checks on a project
ci::run("/path/to/project").unwrap();
```

### Deploying Projects

Cast provides a `deploy` command for deploying Infrastructure as Code (IAC) projects.

```bash
cast deploy
```

This command:
1. If the current project has a `deploys` field in its Cast configuration, it will deploy each project listed in that field
2. Otherwise, verifies the project is marked as `project_type = "iac"` in its Cast configuration
3. Deploys the project based on its framework:
   - **cloudflare-pages**: Deploys using `wrangler pages deploy`
4. Automatically loads environment variables from `.env` file if present (using the `dotenvy` library for proper parsing)
5. Displays deployment progress and output from the deployment tool

**Note**: When deploying from a project with a `deploys` field, the paths are resolved relative to the current project directory. For example, running `cast deploy` from `cookbook/web` with `deploys = ["../cloudflare"]` will deploy the `cookbook/cloudflare` project.

#### Cloudflare Pages Deployment

For Cloudflare Pages projects, the deploy command:
- Checks that `wrangler` is installed
- Verifies that `wrangler.toml` exists in the project directory
- Parses `.env` file using `dotenvy` library (supports escaped characters, quotes, etc.)
- Passes environment variables only to the wrangler command (not set globally)
- Runs `wrangler pages deploy` with inherited stdout/stderr for visibility
- Configuration is read from `wrangler.toml`

The `wrangler.toml` file should contain all deployment configuration including the project name, pages configuration, and build output directory. See [Cloudflare Pages documentation](https://developers.cloudflare.com/pages/configuration/wrangler-configuration/) for details.

Example `.env` file for secrets:
```
CLOUDFLARE_API_TOKEN=your_token_here
CLOUDFLARE_ACCOUNT_ID=your_account_id
# Supports quoted values and special characters
DATABASE_URL="postgresql://user:pass@localhost/db"
```

Example usage in library code:

```rust
use cast_core::deploy;

// Run deploy on an IAC project
deploy::run("/path/to/iac-project").unwrap();
```

### Running CD (Continuous Deployment)

Cast provides a `cd` command for continuous deployment workflows.

```bash
cast cd
```

This command is designed to be called by the Cast CD GitHub workflow when changes are merged. It automatically deploys projects based on the Cast configuration:

1. **Current Project Deployment**: If the current project is an IAC (Infrastructure as Code) project (`project_type = "iac"`), it will be deployed using `cast deploy`.

2. **Deploy Projects**: If the project has a `deploys` list in its Cast configuration, each project in the list will be deployed using `cast deploy`.

This allows you to set up deployment chains where building/updating one project automatically triggers deployment of related infrastructure projects.

#### Deploy Paths

Deploy paths in the `deploys` list are **relative to the current project directory**. This allows for flexible project layouts:

```toml
# Example 1: Deploy a sibling directory
# Project: myapp/web, Deploy: myapp/cloudflare
framework = "dioxus"
deploys = ["../cloudflare"]

# Example 2: Deploy a subdirectory
# Project: myapp, Deploy: myapp/deploy
framework = "dioxus"
deploys = ["deploy"]

# Example 3: Deploy multiple projects
# Project: myapp/web, Deploy: myapp/cloudflare and myapp/auth-cloudflare
framework = "dioxus"
deploys = ["../cloudflare", "../auth-cloudflare"]
```

When you run `cast cd` in a project, it will resolve each path in the `deploys` list relative to the current project directory and deploy those projects.

Example usage in library code:

```rust
use cast_core::cd;

// Run CD on a project
cd::run("/path/to/project").unwrap();
```

### Publishing Artifacts

Cast provides a `publish` command that creates release builds and copies artifacts to a platform-specific directory. The command automatically detects the project type based on the Cast configuration and uses the appropriate build tool.

```bash
cast publish
```

#### For Rust Binary Projects

For standard Rust binaries (projects without a framework configuration or with non-Dioxus frameworks), the command:
1. Runs `cargo build --release` to create an optimized release build
2. Automatically detects the target platform (e.g., `x86_64-unknown-linux-gnu`)
3. Finds the built binary artifact in `target/release`
4. Copies it to `artifacts/<target-triple>/` directory

Example artifact structure:

```bash
# After running cast publish, artifacts are organized by platform:
artifacts/
├── x86_64-unknown-linux-gnu/
│   └── my_binary
├── aarch64-apple-darwin/
│   └── my_binary
└── x86_64-pc-windows-msvc/
    └── my_binary.exe
```

#### For Dioxus Web Projects

For Dioxus web projects (projects with `framework = "dioxus"` in Cast.toml or Cargo.toml), the command:
1. Runs `dx bundle --platform web --release` to create an optimized web bundle
2. Reads the version from `Cargo.toml`
3. Gets the current git commit SHA and checks if the working directory is dirty
4. Generates a timestamped filename: `<version>+<year>-<month>-<day>.<counter>.<sha>[-dirty].zip`
5. Creates a zip file of the bundled assets (excluding `.DS_Store` files)
6. Places the zip in the `artifacts/` directory

Example artifact structure:

```bash
# After running cast publish on a Dioxus project:
artifacts/
└── 0.1.0+2025-01-15.1.a3f4b2c.zip
```

The versioned filename includes:
- **version**: From Cargo.toml `[package] version`
- **date**: Build date in YYYY-MM-DD format
- **counter**: Incremental build counter for the day (stored in `.cast/build_counter_<date>.txt`)
- **sha**: Git commit SHA (truncated to 7 characters)
- **-dirty**: Suffix added if there are uncommitted changes

The build counter is automatically incremented each time `cast publish` is run on the same day, allowing multiple builds per day to have unique filenames.

Example usage in library code:

```rust
use cast_core::publish;

// Build and publish artifacts
publish::run("/path/to/project").unwrap();
```

## Project Management

### Creating New Projects

Cast can create new projects from exemplar projects. Exemplar projects are marked with `exemplar = true` in their `Cast.toml` file.

**Important: Exemplars vs Examples**

- **Exemplar**: Any project in the monorepo marked with `exemplar = true` in its Cast configuration. An exemplar is a good starting point for creating new projects. Exemplars can exist anywhere in the monorepo - they are not limited to a specific directory like "example/".
- **Example**: A workspace or directory (like `example/`) that may contain exemplar projects or demonstration code. The name "example" is just a conventional directory name and has no special meaning to Cast.

Any project can be an exemplar, regardless of where it lives in the repository structure.

```rust
use cast_core::projects;

// Create a new project
projects::new("/path/to/monorepo", "my_project_name").unwrap();
```

This will:
1. Recursively search the entire monorepo for projects marked with `exemplar = true`
2. Copy each exemplar project to the new project location (later exemplars overwrite earlier ones, based on alphabetical ordering)
3. Remove empty `.gitignore` placeholder files used for tracking empty directories in git
4. Remove the `exemplar = true` flag from the new project's Cast.toml
5. Replace `TODO-CHANGE-ME` in `Cargo.toml` package name with the actual project name

The resulting project will have a complete structure ready for development with:
- `Cargo.toml` for Rust dependencies
- `Cast.toml` for Cast-specific configuration
- Standard directories: `src/`, `tests/`, `benches/`, `docs/`, etc.

To create your own exemplar projects, simply add `exemplar = true` to any project's `Cast.toml` file. The Cast tool will find it automatically when creating new projects. You can use `TODO-CHANGE-ME` as a placeholder for the package name in `Cargo.toml`, which will be automatically replaced with the actual project name.

### Finding Projects with Changes

Cast can find projects with changes between two git refs. This is useful for CI/CD workflows to determine which projects need to be tested or built.

```rust
use cast_core::projects;

// Find projects with changes between two commits
let changed_projects = projects::with_changes(
    "/path/to/monorepo",
    "origin/main",  // base ref
    "HEAD"          // head ref
).unwrap();

for project in changed_projects {
    println!("Changed project: {}", project.display());
}
```

This will:
1. Get all changed files between the two git refs using `git diff`
2. Walk up the directory tree from each changed file to find the closest `Cast.toml`
3. Return a sorted, deduplicated list of project directories

The CLI command is available as:
```bash
cast project with-changes --base <base-ref> --head <head-ref>
```

This is used in CI workflows to efficiently run tests only on changed projects.

## Configuration

Cast supports two ways to configure project-specific settings:

1. **Cast.toml** - A dedicated configuration file
2. **Cargo.toml** - Using the `[package.metadata.cast]` section

Cast will automatically check for configuration in Cargo.toml first, then fall back to Cast.toml if no Cast metadata is found. 

**Automatic Defaults for Cargo.toml Projects**: If a project has a Cargo.toml file but no Cast metadata or Cast.toml, Cast will automatically detect it as a Cast project with the following defaults:
- `project_type`: "library" or "binary" (auto-detected based on project structure)
- `language`: "rust"
- `framework`: "cargo"

This means any Rust project with a Cargo.toml is automatically recognized as a Cast project without requiring any additional configuration files.

### Configuration Options

**Option 1: Cast.toml**

```toml
# Whether this project is an exemplar project (example/template)
# Optional: defaults to None/false if not specified
exemplar = true

# Whether this project is a proof of concept project
# Optional: defaults to None/false if not specified
proof_of_concept = true

# The framework used by the project (e.g., "dioxus", "cloudflare-pages", "cargo")
# Optional: defaults to "cargo" for projects with Cargo.toml but no metadata
framework = "dioxus"

# List of projects that are used to deploy this project
# Optional: defaults to None if not specified
deploys = ["deploy-project-1", "deploy-project-2"]

# The type of project (e.g., "static_website", "web_app", "iac", "library", "binary")
# Optional: defaults to "library" or "binary" (auto-detected) for projects with Cargo.toml but no metadata
project_type = "static_website"

# The language of the project (e.g., "rust", "typescript")
# Optional: defaults to "rust" for projects with Cargo.toml but no metadata
language = "rust"
```

**Option 2: Cargo.toml with [package.metadata.cast] section**

```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2021"

[package.metadata.cast]
exemplar = true
proof_of_concept = false
framework = "dioxus"
deploys = ["deploy-project-1", "deploy-project-2"]
project_type = "static_website"
language = "rust"
```

**Option 3: No configuration (auto-detected)**

For any Rust project with a Cargo.toml file, Cast will automatically apply sensible defaults:
- Detects `project_type` as "library" (if `src/lib.rs` exists or `[lib]` section in Cargo.toml) or "binary" (if `src/main.rs` exists or `[[bin]]` section in Cargo.toml)
- Sets `language` to "rust"
- Sets `framework` to "cargo"

### Loading Configuration in Code

```rust
use cast_core::config::CastConfig;

// Load configuration from a directory (checks Cargo.toml first, then Cast.toml, applies defaults if needed)
let config = CastConfig::load_from_dir("path/to/project").unwrap();

// Or load directly from a specific file
let config = CastConfig::load("path/to/Cast.toml").unwrap();
let config = CastConfig::load_from_cargo_toml("path/to/Cargo.toml").unwrap();

// Check if project is an exemplar
if config.exemplar == Some(true) {
    println!("This is an exemplar project");
}

// Check if project is a proof of concept
if config.proof_of_concept == Some(true) {
    println!("This is a proof of concept project");
}

// Check framework
if let Some(framework) = config.framework {
    println!("Framework: {}", framework);
}

// Check deploy projects
if let Some(deploys) = config.deploys {
    println!("Deploy projects: {:?}", deploys);
}

// Check project type
if let Some(project_type) = config.project_type {
    println!("Project type: {}", project_type);
}

// Check language
if let Some(language) = config.language {
    println!("Language: {}", language);
}
```

## Architecture

### Command Pattern

Cast uses the Command/Executor pattern for its CLI commands. This pattern provides a consistent interface for all commands and makes them more testable and maintainable.

#### Command Trait

The `Command` trait defines a common interface that all commands implement:

```rust
use cast_core::command::Command;
use std::path::Path;

pub trait Command {
    fn execute(&self, working_directory: &Path) -> Result<String, Box<dyn std::error::Error>>;
}
```

#### Implementing Commands

Commands are implemented in the `commands` module:

```rust
use cast_core::commands::build::BuildCommand;
use cast_core::command::Command;

let cmd = BuildCommand;
let result = cmd.execute(Path::new("/path/to/project"))?;
println!("{}", result);  // "Build passed"
```

#### Available Command Implementations

Currently implemented:
- `BuildCommand` - Runs cargo build
- `TestCommand` - Runs cargo test
- `CiCommand` - Runs full CI checks (format, lint, build, test)

**In Progress**: Additional commands (Run, Serve, Deploy, Cd, Publish, Session, Project, Toolchain) are being migrated to this pattern. See `cast/ISSUES.md` for the migration roadmap.
