# Priority Issues

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
