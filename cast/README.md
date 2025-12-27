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
use cast::run;

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
use cast::serve;

// Serve static files from a directory
serve::run("/path/to/static/files").unwrap();
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
use cast::build;

// Run build on a project
build::run("/path/to/project").unwrap();
```

### Running Tests

Cast provides a `test` command that runs tests for Rust projects.

```bash
cast test
```

This will run `cargo test` in the current project directory. The command is designed to be consistent with other Cast commands and can be extended in the future with additional test functionality.

Example usage in library code:

```rust
use cast::test;

// Run tests on a project
test::run("/path/to/project").unwrap();
```

### Running CI Checks

Cast provides a `ci` command that runs standard Rust project checks. This is designed to be used in CI workflows.

```bash
cast ci
```

This will run the following checks in order:
1. `cargo fmt --check` - Verify code formatting
2. `cargo clippy -- -D warnings` - Lint code for common mistakes
3. `cast build` - Ensure the project compiles (via `cargo build`)
4. `cast test` - Run all tests (via `cargo test`)

If any check fails, the command will exit with an error. This makes it easy to integrate with CI systems like GitHub Actions.

Example usage in library code:

```rust
use cast::ci;

// Run CI checks on a project
ci::run("/path/to/project").unwrap();
```

### Deploying Projects

Cast provides a `deploy` command for deploying Infrastructure as Code (IAC) projects.

```bash
cast deploy
```

This command:
1. Verifies the project is marked as `project_type = "iac"` in its Cast configuration
2. Deploys the project based on its framework:
   - **cloudflare-pages**: Deploys using `wrangler pages deploy`
3. Automatically loads environment variables from `.env` file if present (using the `dotenvy` library for proper parsing)
4. Displays deployment progress and output from the deployment tool

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
use cast::deploy;

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

Example Cast configuration:

```toml
# A web application that should trigger deployment of its Cloudflare Pages infrastructure
framework = "dioxus"
deploys = ["my-app-cloudflare"]
```

When you run `cast cd` in this project, it will automatically deploy the `my-app-cloudflare` project.

Example usage in library code:

```rust
use cast::cd;

// Run CD on a project
cd::run("/path/to/project").unwrap();
```

## Project Management

### Creating New Projects

Cast can create new projects from exemplar projects. Exemplar projects are marked with `exemplar = true` in their `Cast.toml` file.

**Important: Exemplars vs Examples**

- **Exemplar**: Any project in the monorepo marked with `exemplar = true` in its Cast configuration. An exemplar is a good starting point for creating new projects. Exemplars can exist anywhere in the monorepo - they are not limited to a specific directory like "example/".
- **Example**: A workspace or directory (like `example/`) that may contain exemplar projects or demonstration code. The name "example" is just a conventional directory name and has no special meaning to Cast.

Any project can be an exemplar, regardless of where it lives in the repository structure.

```rust
use cast::projects;

// Create a new project
projects::new("/path/to/monorepo", "my_project_name").unwrap();
```

This will:
1. Recursively search the entire monorepo for projects marked with `exemplar = true`
2. Copy each exemplar project to the new project location (later exemplars overwrite earlier ones, based on alphabetical ordering)
3. Remove empty `.gitignore` placeholder files used for tracking empty directories in git
4. Remove the `exemplar = true` flag from the new project's Cast.toml

The resulting project will have a complete structure ready for development with:
- `Cargo.toml` for Rust dependencies
- `Cast.toml` for Cast-specific configuration
- Standard directories: `src/`, `tests/`, `benches/`, `docs/`, etc.

To create your own exemplar projects, simply add `exemplar = true` to any project's `Cast.toml` file. The Cast tool will find it automatically when creating new projects.

### Finding Projects with Changes

Cast can find projects with changes between two git refs. This is useful for CI/CD workflows to determine which projects need to be tested or built.

```rust
use cast::projects;

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

Cast will automatically check for configuration in Cargo.toml first, then fall back to Cast.toml if no Cast metadata is found. This allows you to consolidate configuration in your existing Cargo.toml file or use a separate Cast.toml file if you prefer.

### Configuration Options

**Option 1: Cast.toml**

```toml
# Whether this project is an exemplar project (example/template)
# Optional: defaults to None/false if not specified
exemplar = true

# Whether this project is a proof of concept project
# Optional: defaults to None/false if not specified
proof_of_concept = true

# The framework used by the project (e.g., "dioxus", "cloudflare-pages", "rust-library")
# Optional: defaults to None if not specified
framework = "dioxus"

# List of projects that are used to deploy this project
# Optional: defaults to None if not specified
deploys = ["deploy-project-1", "deploy-project-2"]

# The type of project (e.g., "static_website", "web_app", "iac", "library", "binary")
# Optional: defaults to None if not specified
project_type = "static_website"
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
```

### Loading Configuration in Code

```rust
use cast::config::CastConfig;

// Load configuration from a directory (checks Cargo.toml first, then Cast.toml)
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
```
