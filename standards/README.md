# Standards

This project provides tooling and documentation for enforcing standards across the monorepo.

## Purpose

The standards project aims to:
- Define and enforce coding standards across all projects
- Provide tooling for validating compliance with monorepo standards
- Document best practices and conventions used in the monorepo
- Automate quality checks and style enforcement

## CLI Tool

The standards CLI provides commands for auditing and enforcing standards across all projects in the monorepo.

### Installation

Build the CLI:

```bash
cargo build --release
```

### Usage

#### Audit Command

Audit projects for standards compliance:

```bash
# Audit from current directory
standards audit

# Audit a specific path
standards audit --path /path/to/monorepo
```

The audit command will discover all projects in the specified path (both Rust and TypeScript projects) and report any violations of defined standards.

**Project Discovery:**
- Automatically finds projects with `Cargo.toml` (Rust) or `package.json` (TypeScript/Node.js)
- Recursively searches directories up to a depth of 10
- Skips common build/dependency directories (`target`, `node_modules`, `.git`)
- Extracts project names and metadata for audit reporting

### Get Help

```bash
# Show available commands
standards --help

# Show help for a specific command
standards audit --help
```

## Standards Documentation

See the `docs/` directory for detailed standards documentation:
- [Naming Conventions](./docs/naming.md)
- [Configuration Standards](./docs/configuration.md)
- [Rust Standards](./docs/rust.md)
- [TypeScript Standards](./docs/typescript.md)
- [Documentation Standards](./docs/documentation.md)
- [Testing Standards](./docs/testing.md)
- [Toolchain Management](./docs/toolchain.md)
- [Build and CI Standards](./docs/build-and-ci.md)
- [Workflow Conventions](./docs/workflow-conventions.md)

## Status

The CLI infrastructure is in place with project discovery implemented. The audit command can now:
- Discover all Rust projects (with `Cargo.toml`)
- Discover all TypeScript/Node.js projects (with `package.json`)
- Report discovered projects with names and paths

See ISSUES.md for planned audit implementations and improvements.

## Future Goals

- Automated linting configuration
- Code formatting standards
- Documentation requirements
- Testing coverage requirements
- Dependency management policies
- CI/CD integration for standard checks
