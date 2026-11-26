# Repository Structure

This document describes the organization and structure of this monorepo to help AI agents and developers understand the codebase.

## Top-Level Structure

```
/
├── .agents/skills/       # Agent Skills (agentskills.io) for AI agents; .claude/skills links here
├── .github/              # GitHub Actions workflows and repo configuration
│   ├── workflows/        # CI/CD and maintenance workflows (pull-request-ci.yml, push-main.yml, nightly-ci.yml, ...)
│   └── dependabot.yml    # Dependency update configuration
├── monorepo/            # High-level monorepo functionality workspace
│   └── workflow_tests/  # Tests for GitHub Actions workflows
├── profiles/            # User profiles and personal content
│   ├── jimhillr/        # Personal profile website for jimhillr
│   └── content_provider/ # Content provider library
├── standards/           # Standards (standards/docs/), shared mise tasks (standards/tasks/) and repo scripts
├── athenaeum/           # Learning resources (algorithms, system design, etc.)
├── [other projects]     # Various projects and applications
├── mise.toml            # mise monorepo root: lists projects, //:audit and //:affected tasks
├── AGENTS.md            # Repo-wide instructions for AI agents (CLAUDE.md imports it)
├── ISSUES.md            # Project-wide TODO and issue tracking
├── HUMAN_ISSUES.md      # Tasks only a person can do (TODO (human))
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
Contains the repository's standards and conventions in `standards/docs/` (Rust, testing, documentation, issue management, workflow conventions, toolchain, mise), the shared task files every project includes (`standards/tasks/`), and the scripts behind `//:audit` and `//:affected` (`standards/scripts/`).

### `/athenaeum/`
Contains learning resources (algorithms, system design, language notes, etc.).

There is no top-level `docs/` folder. The root `README.md` is a regular file, and project-specific docs live inside each project (for example `cookbook/README.md`).

### Project Directories
All projects live in the root directory. Each project may have:
- `mise.toml` - Tools and the standard tasks (includes one `standards/tasks/<type>.toml`)
- `Cargo.toml` - Rust project configuration
- Standard Rust project structure: `src/`, `tests/`, `benches/`, etc.

### `.github/workflows/`
Contains GitHub Actions workflows, including:
- `pull-request-ci.yml` - Runs `mise run ci` for projects changed in a pull request
- `push-main.yml` - CI and deploy for projects changed on main
- `nightly-ci.yml` - Runs `mise run ci` in every project each night
- `mise-projects.yml` - The reusable workflow the three above call
- `all-delete-branches.yml` - Deletes branches (manual trigger, dry run by default)
- `all-codeql.yml` - CodeQL code scanning

See `.github/workflows/README.md` and `standards/docs/workflow-conventions.md` for details.

## Finding Issues/TODOs

1. Check `ISSUES.md` in the repository root and in each project
2. Use the TODO Tree extension to find TODOs in code
3. AI agents work on TODOs from ISSUES.md using the `start-a-new-task` skill in `.agents/skills/`

## Project Dependencies

- **Rust**: Primary language for most projects
- **Cargo**: Rust package manager
- Projects may have language-specific dependencies (npm, etc.)

## Testing and CI

- Each project can be tested with `mise run ci` in its directory
- The Pull Request CI workflow automatically validates changed projects on PRs
- The push-main workflow validates and deploys changed projects on main branch commits
- Tests should be run before committing changes
- Workflow tests are located at `monorepo/workflow_tests/` and validate GitHub Actions configurations

## Important Notes for Agents

1. **Project structure**: All projects are located in the root directory, not under a "projects/" subdirectory
2. **Docs location**: There is no root `/docs/` folder. Standards live in `/standards/docs/`; project docs live inside each project
3. **Minimal workflow logic**: GitHub workflows should call mise tasks, not contain complex logic
4. **No project templates**: There are no exemplar/template projects, and they are intentionally not being restored. Create new projects by hand
5. **TODO tracking**: Remove TODOs from ISSUES.md when completed
6. **Concurrency**: Only one automated agent task should be open at a time (an open agent PR means wait)
7. **mise**: Every project has a `mise.toml` that includes one shared task file; `mise run //:audit` enforces this.
