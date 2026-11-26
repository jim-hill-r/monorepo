# Repository Structure

This document describes the organization and structure of this monorepo to help GitHub Copilot agents and developers understand the codebase.

## Top-Level Structure

```
/
├── .github/              # GitHub Actions workflows and agent configurations
│   ├── workflows/        # CI/CD and maintenance workflows (pull-request-ci.yml, push-main-ci.yml, nightly-ci.yml, ...)
│   ├── copilot-instructions.md # Instructions for coding agents
│   └── dependabot.yml    # Dependency update configuration
├── monorepo/            # High-level monorepo functionality workspace
│   └── workflow_tests/  # Tests for GitHub Actions workflows
├── profiles/            # User profiles and personal content
│   ├── jimhillr/        # Personal profile website for jimhillr
│   └── content_provider/ # Content provider library
├── standards/           # Coding, testing, workflow and documentation standards (standards/docs/)
├── athenaeum/           # Learning resources (algorithms, system design, etc.)
├── cast/               # Cast monorepo tooling workspace (contains core, cli, vscode_ext)
├── agent_copilot/       # Binary for creating GitHub Copilot agent tasks
│   └── prompts/         # Prompts for GitHub Copilot agents
├── [other projects]     # Various projects and applications
├── ISSUES.md            # Project-wide TODO and issue tracking
├── REPOSITORY_STRUCTURE.md # This file
└── README.md            # Main README
```

## Key Directories

### `/monorepo/`
Contains high-level monorepo functionality and tools:
- `workflow_tests/` - Rust-based tests for GitHub Actions workflows

### `/profiles/`
Contains:
- User profiles and personal content
- Personal profile websites (e.g., jimhillr)
- Content provider library for generating profile content

### `/standards/`
Contains the repository's standards and conventions in `standards/docs/` (Rust, testing, documentation, issue management, workflow conventions, toolchain).

### `/athenaeum/`
Contains learning resources (algorithms, system design, language notes, etc.).

There is no top-level `docs/` folder. The root `README.md` is a regular file, and project-specific docs live inside each project (for example `cast/core/docs/`).

### Project Directories
All projects live in the root directory. Each project may have:
- `Cast.toml` - Cast configuration (marks proof-of-concepts, frameworks, etc.)
- `Cargo.toml` - Rust project configuration
- Standard Rust project structure: `src/`, `tests/`, `benches/`, etc.

### `.github/workflows/`
Contains GitHub Actions workflows, including:
- `pull-request-ci.yml` - Runs `cast ci` for projects changed in a pull request
- `push-main-ci.yml` / `push-main-cd.yml` - CI and delivery for commits on main
- `nightly-ci.yml` / `nightly-standards-audit.yml` - Scheduled CI and standards audit
- `all-start-a-new-task.yml` - Creates a new agent task (manual trigger only)
- `all-delete-branches.yml` - Deletes branches (manual trigger, dry run by default)
- `all-codeql.yml` - CodeQL code scanning

See `.github/workflows/README.md` and `.github/WORKFLOW_CONVENTIONS.md` for details.

## Cast Workspace

The Cast monorepo tooling is organized as a Cargo workspace at `cast/`:
- **core** (`cast_core`) - Core Cast library for monorepo operations
- **cli** (`cast_cli`) - Cast command-line interface
- **vscode_ext** (`cast`) - VSCode extension for Cast

### Cast CLI

The `cast` CLI is the primary tool for managing this monorepo:
- Located at `cast/cli/`
- Build with: `cd cast/cli && cargo build --release`
- Commands:
  - `cast project new <name>` - Create a new project by copying exemplar projects (`exemplar = true` in `Cast.toml`). The repository currently has no exemplar projects, so this command fails with "No exemplar projects found".
  - `cast project with-changes --base <ref> --head <ref>` - Find projects with changes between git refs
  - `cast ci` - Run CI checks (lint, build, test)
  - `cast session start` - Start a work session
  - `cast install` - Install required tools for projects

## Finding Issues/TODOs

1. Check `ISSUES.md` in the repository root and in each project
2. Use the TODO Tree extension to find TODOs in code
3. GitHub Copilot agents automatically work on TODOs from ISSUES.md

## Project Dependencies

- **Rust**: Primary language for most projects
- **Cargo**: Rust package manager
- Projects may have language-specific dependencies (npm, etc.)

## Testing and CI

- Each project with `Cast.toml` can be tested with `cast ci`
- The Pull Request CI workflow automatically validates changed projects on PRs
- The push-main CI/CD workflows validate and deliver changed projects on main branch commits
- Tests should be run before committing changes
- Workflow tests are located at `monorepo/workflow_tests/` and validate GitHub Actions configurations

## Important Notes for Agents

1. **Project structure**: All projects are located in the root directory, not under a "projects/" subdirectory
2. **Docs location**: There is no root `/docs/` folder. Standards live in `/standards/docs/`; project docs live inside each project
3. **Minimal workflow logic**: GitHub workflows should call `cast` commands, not contain complex logic
4. **No project templates**: There are no exemplar/template projects, and they are intentionally not being restored. Create new projects by hand
5. **TODO tracking**: Remove TODOs from ISSUES.md when completed
6. **Concurrency**: Only one agent task should run at a time (handled by workflows)
7. **Cast Tooling**: All projects should have a Cast.toml if it is missing.
