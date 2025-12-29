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

The audit command will check all projects against defined standards and report any violations.

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
- [Toolchain Management](./docs/toolchain.md)

## Status

The CLI infrastructure is in place. See ISSUES.md for planned audit implementations and improvements.

## Future Goals

- Automated linting configuration
- Code formatting standards
- Documentation requirements
- Testing coverage requirements
- Dependency management policies
- CI/CD integration for standard checks
