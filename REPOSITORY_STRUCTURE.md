# Repository Structure

This document describes the organization and structure of this monorepo to help GitHub Copilot agents and developers understand the codebase.

## Top-Level Structure

```
/
├── .github/              # GitHub Actions workflows and agent configurations
│   ├── workflows/        # CI/CD workflows (cast-ci.yml, start-a-new-task.yml)
│   └── dependabot.yml    # Dependency update configuration
├── macos/               # macOS setup guide and instructions
├── profiles/            # User profiles and personal content
│   ├── jimhillr/        # Personal profile website for jimhillr
│   └── content_provider/ # Content provider library
├── docs/                # Documentation and learning resources
├── cast/                # Core Cast library for monorepo tooling
├── cast_cli/            # Cast command-line interface
├── agent-copilot/       # Binary for creating GitHub Copilot agent tasks
│   └── prompts/         # Prompts for GitHub Copilot agents
├── base/                # Exemplar project with base configuration
├── library/             # Exemplar project for libraries
├── binary/              # Exemplar project for binaries
├── [other projects]     # Various projects and applications
├── ISSUES.md            # Project-wide TODO and issue tracking
└── README.md            # Main README (symlink to docs/README.md)
```

## Key Directories

### `/macos/`
Contains setup guide and instructions for configuring a new macOS machine with the required global dependencies for this monorepo (Rust, npm, etc.).

### `/profiles/`
Contains:
- User profiles and personal content
- Personal profile websites (e.g., jimhillr)
- Content provider library for generating profile content

### `/docs/`
Contains:
- Documentation and guides
- Learning resources (algorithms, system design, etc.)
- Standards and conventions
- The main repository README.md

The root-level `README.md` is a symlink pointing to `./docs/README.md`.

### Project Directories
All projects live in the root directory. Each project may have:
- `Cast.toml` - Cast configuration (marks exemplar projects, proof-of-concepts)
- `Cargo.toml` - Rust project configuration
- Standard Rust project structure: `src/`, `tests/`, `benches/`, etc.

### `.github/workflows/`
Contains GitHub Actions workflows:
- `cast-ci.yml` - Automatically runs `cast ci` for projects with changes
- `start-a-new-task.yml` - Creates new agent tasks after PR merges

See `.github/workflows/README.md` and `.github/WORKFLOW_CONVENTIONS.md` for details.

## Cast CLI

The `cast` CLI is the primary tool for managing this monorepo:
- Located at `cast_cli/`
- Build with: `cargo build --release`
- Commands:
  - `cast project new <name>` - Create new project from exemplars
  - `cast project with-changes --base <ref> --head <ref>` - Find projects with changes between git refs
  - `cast ci` - Run CI checks (lint, build, test)
  - `cast session start` - Start a work session

## Exemplar Projects

Projects marked with `exemplar = true` in `Cast.toml` serve as templates:
- `base/` - Basic project structure
- `library/` - Rust library template
- `binary/` - Rust binary/CLI template

When creating a new project, Cast copies exemplar projects in alphabetical order, with later ones overwriting files from earlier ones.

## Finding Issues/TODOs

1. Check `ISSUES.md` in the repository root
2. Use the TODO Tree extension to find TODOs in code
3. GitHub Copilot agents automatically work on TODOs from ISSUES.md

## Project Dependencies

- **Rust**: Primary language for most projects
- **Cargo**: Rust package manager
- Projects may have language-specific dependencies (npm, etc.)

## Testing and CI

- Each project with `Cast.toml` can be tested with `cast ci`
- The Cast CI workflow automatically detects and tests changed projects
- Tests should be run before committing changes

## Important Notes for Agents

1. **Project structure**: All projects are located in the root directory, not under a "projects/" subdirectory
2. **Docs location**: The docs folder is in the root directory at `/docs/`
3. **Minimal workflow logic**: GitHub workflows should call `cast` commands, not contain complex logic
4. **Exemplar projects**: Base, library, and binary are templates - don't modify unless necessary
5. **TODO tracking**: Remove TODOs from ISSUES.md when completed
6. **Concurrency**: Only one agent task should run at a time (handled by workflows)
7. **Cast Tooling**: All projects should have a Cast.toml if it is missing.
