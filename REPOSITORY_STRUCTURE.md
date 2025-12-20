# Repository Structure

This document describes the organization and structure of this monorepo to help GitHub Copilot agents and developers understand the codebase.

## Top-Level Structure

```
/
├── .github/              # GitHub Actions workflows and agent configurations
│   ├── workflows/        # CI/CD workflows (cast-ci.yml, start-a-new-task.yml)
│   ├── agent-prompts/    # Prompts for GitHub Copilot agents
│   └── dependabot.yml    # Dependency update configuration
├── projects/             # All projects in the monorepo
│   ├── docs/            # Documentation and learning resources (moved here from root)
│   ├── cast/            # Core Cast library for monorepo tooling
│   ├── cast_cli/        # Cast command-line interface
│   ├── agent-copilot/   # Binary for creating GitHub Copilot agent tasks
│   ├── base/            # Exemplar project with base configuration
│   ├── library/         # Exemplar project for libraries
│   ├── binary/          # Exemplar project for binaries
│   └── [other projects] # Various projects and applications
├── proof_of_concepts/    # Proof-of-concept experiments
├── ISSUES.md            # Project-wide TODO and issue tracking
└── README.md            # Main README (symlink to projects/docs/README.md)
```

## Key Directories

### `/projects/`
All projects live in this directory. Each project may have:
- `Cast.toml` - Cast configuration (marks exemplar projects, proof-of-concepts)
- `Cargo.toml` - Rust project configuration
- Standard Rust project structure: `src/`, `tests/`, `benches/`, etc.

### `/projects/docs/`
**Important: As of December 2024, the docs folder was moved from root into projects.**

Contains:
- Documentation and guides
- Learning resources (algorithms, system design, etc.)
- Standards and conventions
- The main repository README.md

The root-level `README.md` is a symlink pointing to `./projects/docs/README.md`.

### `.github/workflows/`
Contains GitHub Actions workflows:
- `cast-ci.yml` - Automatically runs `cast ci` for projects with changes
- `start-a-new-task.yml` - Creates new agent tasks after PR merges

See `.github/workflows/README.md` and `.github/WORKFLOW_CONVENTIONS.md` for details.

## Cast CLI

The `cast` CLI is the primary tool for managing this monorepo:
- Located at `projects/cast_cli/`
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

1. **Docs location**: The docs folder is in `projects/docs/`, not at the root level
2. **Minimal workflow logic**: GitHub workflows should call `cast` commands, not contain complex logic
3. **Exemplar projects**: Base, library, and binary are templates - don't modify unless necessary
4. **TODO tracking**: Remove TODOs from ISSUES.md when completed
5. **Concurrency**: Only one agent task should run at a time (handled by workflows)
