# Priority Issues

- TODO (agent-generated): Make the git hooks usable in cloud sandboxes (Jim, 2026-11-26: hooks were skipped with `LEFTHOOK=0` on PR #582 because of this). Today a commit that only touches a root Markdown file such as ISSUES.md, or a merge from main, makes `hooks.sh` run `mise run ci` in about every project (8 to 14 minutes), and four projects can't build in the sandbox (my_lang needs LLVM headers, controlled_chaos and starcraft need alsa, luggage needs utoipa-swagger-ui downloads; the Playwright step of cahokia/web and cookbook/web already skips itself there), so the commit is refused. Fix: on a merge commit, check only the projects the branch itself changed since the merge base, not main's incoming changes; don't treat root Markdown files as touching every project; and when a project's build fails only because a system tool or download is missing, have the hook install it through the project's `ci:deps` task or report it as "not checked here, PR CI will" instead of failing. Add tests for each case.
- TODO: Port the checks the removed `standards` CLI ran into `standards/scripts/audit.sh`: NAM-002 (directory matches package name), NAM-004 (`poc_` prefix), DOC-001 to DOC-004 (README.md and CONTRIBUTING.md with a Getting Started section) and LIN-001/LIN-002 (Cargo lints). About 20 projects miss a README.md or CONTRIBUTING.md today, so add them as warnings first, or fix the projects in the same PR.

# Backlog

## Standards (broken down)

- TODO: Create Documentation Standards - Establish documentation requirements and tooling: README.md template for new projects, API documentation standards (rustdoc), Architecture decision records (ADRs), Changelog format and maintenance.
- TODO: Implement Testing Standards - Define testing requirements and coverage goals: minimum code coverage thresholds, testing framework conventions, integration test patterns, benchmark standards.
- TODO: Dependency Management Policy - Create policies for managing dependencies: approved dependency list, version management strategy, security vulnerability scanning, license compliance checking.
- TODO: CI/CD Integration - Integrate standards enforcement into CI/CD pipelines: GitHub Actions workflows, automated code review checks, quality gates for PR merges.
- TODO: Pre-commit hooks. `mise run ci` for the changed projects (see `mise run //:affected`) should run as a part of pre-commit.
- TODO: Code Style Guidelines - Document and enforce code style conventions: naming conventions, module organization patterns, error handling patterns, async/await best practices.
- TODO: Build and Release Standards - Standardize build and release processes. Versioning strategy (semver): we use semver plus datetimes as a build identifier. We should remove any incrementing build counters.

# Priority Projects

# On Hold Projects
