# Priority Issues

# Backlog

## Standards CLI Implementation (Broken Down)

# Other Issues
## TODO: Implement Linting Standards
TODO: Define and enforce linting standards across all Rust projects in the monorepo. Consider:
- Clippy configuration (clippy.toml)
- Rustfmt configuration (rustfmt.toml)
- Custom lint rules for monorepo-specific patterns

## TODO: Create Documentation Standards
TODO: Establish documentation requirements and tooling:
- README.md template for new projects
- API documentation standards (rustdoc)
- Architecture decision records (ADRs)
- Changelog format and maintenance

## TODO: Implement Testing Standards
TODO: Define testing requirements and coverage goals:
- Minimum code coverage thresholds
- Testing framework conventions
- Integration test patterns
- Benchmark standards

## TODO: Dependency Management Policy
TODO: Create policies for managing dependencies:
- Approved dependency list
- Version management strategy
- Security vulnerability scanning
- License compliance checking

## TODO: CI/CD Integration
TODO: Integrate standards enforcement into CI/CD pipelines:
- TODO: Pre-commit hooks. cast ci --only-changed --check --recursive 2 should run as a part of pre-commit.
- GitHub Actions workflows
- Automated code review checks
- Quality gates for PR merges

## TODO: Code Style Guidelines
TODO: Document and enforce code style conventions:
- Naming conventions
- Module organization patterns
- Error handling patterns
- Async/await best practices

## TODO: Build and Release Standards
TODO: Standardize build and release processes (this is all in process)
- Build configuration templates
- TODO: Versioning strategy (semver) (we use semver plus datetimes as a build identifier. We should remove any incrementing build counters)
- Release automation
- Artifact publishing guidelines
