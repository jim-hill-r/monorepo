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

#### Add TODO Command

Add TODO items to ISSUES.md files:

```bash
# Add a TODO to Priority Issues section (default)
standards add-todo --todo "TODO: Implement feature X"

# Add a TODO to a specific ISSUES.md file
standards add-todo --file /path/to/ISSUES.md --todo "TODO: Fix bug Y"

# Add a TODO to Backlog section
standards add-todo --todo "TODO: Consider optimization Z" --backlog
```

The add-todo command will:
- Create ISSUES.md if it doesn't exist
- Parse existing ISSUES.md and preserve its structure
- Check for duplicate TODOs (case-insensitive)
- Add the TODO to the appropriate section (Priority Issues or Backlog)
- Maintain proper formatting and section headers

#### Audit-to-Issues Command

Audit projects and automatically write violations as TODO entries to their ISSUES.md files:

```bash
# Audit from current directory and write violations to ISSUES.md files
standards audit-to-issues

# Audit a specific path
standards audit-to-issues --path /path/to/monorepo

# Run without summary report
standards audit-to-issues --summary false
```

The audit-to-issues command will:
- Discover all projects in the specified path
- Run all standards audits (naming, configuration, documentation)
- Convert violations to TODO entries with the format: `TODO (agent-generated): [STANDARD-ID] project_name - message`
- Write TODOs to each project's ISSUES.md file in the Priority Issues section
- Skip duplicate TODOs (case-insensitive comparison)
- Generate a summary report showing how many TODOs were written to each project

This command is useful for:
- Batch processing audit results across all projects
- Automatically creating actionable TODO items from violations
- Tracking standards compliance issues in each project's ISSUES.md

### Get Help

```bash
# Show available commands
standards --help

# Show help for a specific command
standards audit --help
standards add-todo --help
standards audit-to-issues --help
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

The CLI infrastructure is in place with the following features implemented:

### Implemented Audits

#### Naming Standards Audit ✅
The naming standards audit is fully implemented and checks:
- **NAM-001**: All projects MUST be snake_case
- **NAM-002**: Directory name must match package name
- **NAM-004**: Proof of Concept projects must begin with `poc_`

Example output:
```
Standards audit for path: .

Discovered 35 project(s)

=== Naming Standards Violations ===

[NAM-001] InvalidName - Project name 'InvalidName' is not in snake_case format. All projects MUST be snake_case.
  Path: ./example/InvalidName
  Severity: Error

[NAM-002] correct_name - Directory name 'wrong_dir' does not match package name 'correct_name'. Directory name MUST match the package name.
  Path: ./example/wrong_dir
  Severity: Error

Total violations: 2
```

#### Documentation Standards Audit ✅
The documentation standards audit is fully implemented and checks:
- **DOC-001**: All projects MUST include a README.md (Error)
- **DOC-002**: README.md should include a section with the project name and description (Warning)
- **DOC-003**: All projects MUST include a CONTRIBUTING.md (Error)
- **DOC-004**: CONTRIBUTING.md should include a "Getting Started" section describing how to install toolchain, build, and test (Warning)

Example output:
```
=== Documentation Standards Violations ===

[DOC-001] myproject - Project 'myproject' is missing README.md. All projects MUST include a README.md.
  Path: ./myproject
  Severity: Error

[DOC-003] myproject - Project 'myproject' is missing CONTRIBUTING.md. All projects MUST include a CONTRIBUTING.md.
  Path: ./myproject
  Severity: Error

Total violations: 2
```

### Project Discovery
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
