# Priority Issues

## Standards CLI Implementation (Broken Down)

### TODO (agent-generated): Create standards CLI binary skeleton
Create a new binary crate for the standards CLI with basic argument parsing:
- Add `[[bin]]` section to Cargo.toml
- Implement basic CLI structure with clap
- Add `audit` subcommand skeleton
- Write basic integration tests

### TODO (agent-generated): Implement standards definition framework
Create a framework for defining and loading standards:
- Define `Standard` trait for different standard types
- Implement standard loading from docs/ directory
- Create parser for standards documentation
- Add unit tests for standard parsing

### TODO (agent-generated): Implement project discovery
Add functionality to discover projects in the monorepo:
- Traverse directory structure
- Identify projects (Cargo.toml, package.json, etc.)
- Collect project metadata
- Test with sample directory structures

### TODO (agent-generated): Implement naming standards audit
Create auditing logic for naming standards (standards/docs/naming.md):
- Check project names are snake_case
- Verify directory name matches package name
- Validate deployment naming conventions
- Validate PoC project naming

### TODO (agent-generated): Implement configuration standards audit
Create auditing logic for configuration standards (standards/docs/configuration.md):
- Check for royalty.toml presence
- Validate configuration file format
- Report missing configuration files

### TODO (agent-generated): Implement documentation standards audit
Create auditing logic for documentation standards (standards/docs/documentation.md):
- Check for README.md presence
- Validate README.md sections
- Check for CONTRIBUTING.md presence
- Validate CONTRIBUTING.md sections

### TODO (agent-generated): Implement ISSUES.md modification
Add functionality to add TODOs to project ISSUES.md files:
- Parse existing ISSUES.md files
- Add new TODO entries without duplicates
- Maintain proper formatting
- Create ISSUES.md if it doesn't exist

### TODO (agent-generated): Integrate audit results with ISSUES.md
Connect the audit engine with ISSUES.md modification:
- Convert audit findings to TODO entries
- Write TODOs to appropriate project ISSUES.md files
- Generate summary report
- Add end-to-end integration tests

### TODO (agent-generated): Add CLI documentation
Document the standards CLI tool:
- Add usage examples to standards/README.md
- Document CLI commands and options
- Add troubleshooting guide
- Include example audit output

# Other Issues
## TODO: Implement Linting Standards
Define and enforce linting standards across all Rust projects in the monorepo. Consider:
- Clippy configuration (clippy.toml)
- Rustfmt configuration (rustfmt.toml)
- Custom lint rules for monorepo-specific patterns

## TODO: Create Documentation Standards
Establish documentation requirements and tooling:
- README.md template for new projects
- API documentation standards (rustdoc)
- Architecture decision records (ADRs)
- Changelog format and maintenance

## TODO: Implement Testing Standards
Define testing requirements and coverage goals:
- Minimum code coverage thresholds
- Testing framework conventions
- Integration test patterns
- Benchmark standards

## TODO: Dependency Management Policy
Create policies for managing dependencies:
- Approved dependency list
- Version management strategy
- Security vulnerability scanning
- License compliance checking

## TODO: CI/CD Integration
Integrate standards enforcement into CI/CD pipelines:
- Pre-commit hooks
- GitHub Actions workflows
- Automated code review checks
- Quality gates for PR merges

## TODO: Code Style Guidelines
Document and enforce code style conventions:
- Naming conventions
- Module organization patterns
- Error handling patterns
- Async/await best practices

## TODO: Build and Release Standards
Standardize build and release processes:
- Build configuration templates
- Versioning strategy (semver)
- Release automation
- Artifact publishing guidelines
