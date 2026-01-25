# Cast Workspace

This workspace contains the Cast monorepo tooling projects. Cast provides highly opinionated tooling for Rust monorepos, making it simple to manage, build, test, and deploy projects locally, on CI, and in cloud environments.

A cast is a [group of crabs](https://www.originaldiving.com/blog/our-favourite-collective-nouns-for-sea-creatures#:~:text=A%20group%20of%20crabs%20is,crabs%20dominating%20access%20to%20food.).

## Workspace Members

This workspace is organized into three main components:

### Core Library (`cast_core`)

The core Cast library provides the foundational functionality for monorepo operations.

**Location**: `cast/core/`

**Key Features**:
- Project detection and configuration management (Cast.toml, Cargo.toml metadata)
- CI/CD operations (build, test, format, lint)
- Development server management (run, serve)
- Tool management (install, check, list development tools)
- Deployment operations (Cloudflare Pages, SSG bundles)
- Project dependency analysis

**Documentation**: See [core/README.md](core/README.md) for complete API documentation and usage examples.

### Command-Line Interface (`cast_cli`)

The Cast CLI is the primary tool for developers working with Cast-enabled monorepos.

**Location**: `cast/cli/`

**Key Commands**:
- `cast ci` - Run CI checks (format, lint, build, test)
  - `cast ci --check` - Run checks only without creating artifacts (default mode for PR validation)
  - `cast ci --fix` - Auto-fix formatting issues, then run checks without creating artifacts
  - `cast ci --release` - Build in release mode, publish artifacts, and commit them (for post-merge to master)
  - `cast ci --recursive <depth>` - After running CI, find and run CI on cast projects up to N levels below the current directory
  - `cast ci --only-changed` - Only run CI if the project has changes compared to the origin's default branch
- `cast dev` - Start development server (auto-detects framework)
- `cast serve` - Serve static files for testing
- `cast build` - Build projects
- `cast test` - Run tests
- `cast publish` - Build release artifacts and copy to artifacts directory
- `cast install` - Install and manage framework-specific development tools
- `cast uninstall` - Uninstall cast-managed tools
- `cast cd` - Deploy to production environments
- `cast project` - Analyze project dependencies and changes

**Installation**: See the Installation section below.

### VSCode Extension (`cast`)

The VSCode extension provides IDE integration for Cast workflows.

**Location**: `cast/vscode_ext/`

**Status**: This is a TypeScript/Node.js project, not part of the Cargo workspace. It provides Cast command integration directly in VSCode.

**Documentation**: See [vscode_ext/README.md](vscode_ext/README.md) for extension details.

## Installation

### Prerequisites

- Rust (rustc, cargo, rustfmt, clippy)
- For framework-specific projects, additional tools may be required (see Tool Management)

### Installing Cast CLI

From the workspace root:

```bash
# Build the CLI in release mode
cargo build --release -p cast_cli

# The binary will be at cast/cli/target/release/cast
# You can add it to your PATH or use it directly
./cli/target/release/cast --help
```

Or install it to your Cargo bin directory:

```bash
cd cli
cargo install --path .
cast --help
```

## Building

Build all Rust workspace members:
```bash
cargo build --workspace
```

Build specific member:
```bash
cargo build -p cast_core
cargo build -p cast_cli
```

Build with optimizations:
```bash
cargo build --workspace --release
```

## Testing

Run all tests in the workspace:
```bash
cargo test --workspace
```

Test specific member:
```bash
cargo test -p cast_core
cargo test -p cast_cli
```

Run tests with output:
```bash
cargo test --workspace -- --nocapture
```

## Development

### Using Cast CLI

The Cast CLI is the primary interface for monorepo operations:

```bash
# Run CI checks on current project (includes format, lint, build, test, and publish)
cd cli
cargo build --release

# Run CI with default check mode
./target/release/cast ci

# Run CI with auto-fix mode (auto-formats code)
./target/release/cast ci --fix

# Run CI with release mode (build --release and publish artifacts)
./target/release/cast ci --release

# Run CI recursively on child projects
# This runs CI on the current project, then finds and runs CI on all cast projects
# up to 2 levels below the current directory
./target/release/cast ci --recursive 2

# Start development server (auto-detects framework)
./target/release/cast dev

# Install required development tools
./target/release/cast install

# Check which tools are installed
./target/release/cast install check

# Build release artifacts
./target/release/cast publish
```

### Only Changed CI Example

The `--only-changed` option is useful in CI/CD pipelines to skip projects without changes:

```bash
# Only run CI if the project has changes compared to origin's default branch
./target/release/cast ci --only-changed

# Combine with recursive to check all projects, but only run CI on changed ones
./target/release/cast ci --recursive 2 --only-changed
```

This feature is helpful in monorepos where you want to optimize CI runtime by only running checks on projects that have actually changed since the last deployment to the default branch.

**Performance optimization:** When using `--only-changed` with `--recursive`, Cast caches git diff results to avoid running the same git commands multiple times, making recursive traversal very fast when most projects have no changes.

### Recursive CI Example

The `--recursive` option is useful for monorepos with nested projects:

```
monorepo/
├── Cast.toml
├── Cargo.toml
├── project1/
│   ├── Cast.toml
│   └── Cargo.toml
└── project2/
    ├── Cast.toml
    └── Cargo.toml
```

Running `cast ci --recursive 1` from the monorepo root will:
1. Run CI on the root project
2. Find project1 and project2 at depth 1
3. Run CI on each discovered project

**Note**: If the current directory doesn't have a Cast configuration (Cast.toml or Cargo.toml with Cast metadata), the recursive flag will skip running CI on the current directory and only search for and run CI on child projects. This is useful for running CI across multiple projects from a parent directory that isn't itself a Cast project.

```bash
# From a directory without Cast.toml, find and run CI on all projects within 2 levels
cast ci --recursive 2
```

This ensures all projects in your monorepo pass CI checks without manually running CI in each directory.

### Working on Cast Core

The core library is used by the CLI and can also be used programmatically:

```rust
use cast_core::ci;

// Run CI checks on a project with default Check mode
ci::run("/path/to/project", ci::CiMode::Check).unwrap();

// Run CI checks with Fix mode (auto-format code)
ci::run("/path/to/project", ci::CiMode::Fix).unwrap();

// Run CI checks with Release mode (build --release)
ci::run("/path/to/project", ci::CiMode::Release).unwrap();
```

See [core/README.md](core/README.md) for comprehensive API documentation and examples.

### Working on VSCode Extension

The VSCode extension is a separate Node.js project:

```bash
cd vscode_ext
npm install
npm run compile
# See vscode_ext/README.md for development instructions
```

## Tool Management

Cast provides tool management to help install and uninstall framework-specific tools:

```bash
# Install all required tools for a project
cast install

# Uninstall cast-managed tools
cast uninstall --all

# Uninstall a specific tool
cast uninstall --tool dx

# Check if tools are installed
cast install check

# See what would be installed/uninstalled
cast install --dry-run
cast uninstall --dry-run --all
```

Different frameworks require different tools:
- **Dioxus**: Requires `dx` CLI, Node.js, npm, and Playwright
- **Cloudflare Pages**: Requires Wrangler CLI
- **Pure Rust**: Only requires Rust

**Note**: Cast can only uninstall tools that it installed via cargo or npm (e.g., dx, playwright, wrangler). System-level tools like Node.js, rustc, and git-lfs must be managed separately.

See [core/README.md](core/README.md) for complete tool management documentation.

## Workspace Structure

```
cast/
├── Cargo.toml          # Workspace configuration
├── Cast.toml           # Cast workspace configuration
├── README.md           # This file
├── ISSUES.md           # Tracked issues and development tasks
├── core/               # Cast core library (cast_core)
│   ├── src/            # Core functionality
│   ├── tests/          # Integration tests
│   ├── benches/        # Performance benchmarks
│   ├── examples/       # Usage examples
│   └── docs/           # Design documents
├── cli/                # Cast CLI (cast_cli)
│   ├── src/            # CLI implementation
│   └── examples/       # CLI usage examples
└── vscode_ext/         # VSCode extension (cast)
    ├── src/            # Extension TypeScript code
    └── package.json    # Node.js dependencies
```

## Documentation

- **Core Library**: [core/README.md](core/README.md) - Complete API documentation
- **Install Command Design**: [core/docs/install-command-design.md](core/docs/install-command-design.md) - Tool management specification
- **Workspace Issues**: [ISSUES.md](ISSUES.md) - Development roadmap and tracked tasks

## Contributing

When working on Cast:

1. Make changes to the appropriate workspace member
2. Run tests: `cargo test --workspace`
3. Run formatting: `cargo fmt --workspace`
4. Run lints: `cargo clippy --workspace`
5. Build in release mode to test: `cargo build --workspace --release`

The Cast workspace follows the repository's [Rust coding standards](../standards/docs/rust.md).
