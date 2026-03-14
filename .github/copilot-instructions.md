# Standards

All coding standards are found in the `standards` project in this monorepo. Reference those standards as primary source of truth. If duplicate or contradictory information exists in this document, copilot should remove the information here and instead recommend referencing the information there. If information is missing there that copilot is using from here, it should be added to the `standards` project and removed from here.

## Rust Code Standards

See `standards/docs/rust.md` for complete Rust coding standards and proper error handling patterns.

## Issue Management

See `standards/docs/issue-management.md` for complete issue management standards and conventions.

## Branching Requirements
All branches should have unique names to prevent collisions.

- Include a timestamp in the branch name.

## Testing Requirements

See `standards/docs/testing.md` for complete testing standards and requirements.

All code changes require appropriate tests:
1. **Unit Tests**: All code changes must include unit tests
   - For Rust code: Add `#[cfg(test)]` modules with test functions
   - Run tests with `cargo test` in the project directory
2. **Playwright Tests**: All UI changes must include Playwright end-to-end tests
   - For Dioxus web applications: Add `.spec.ts` files in the `tests/` directory
   - Run tests with `npm test` after starting the dev server
   - See existing test files for examples of patterns and best practices

## UI Changes Documentation

See `standards/docs/documentation.md` for complete UI change documentation standards including screenshot requirements.

## Task Completion Requirements

Before finishing any task that involves code changes:
1. Always run `cast ci` on any projects that have been modified
2. Ensure `cast ci` passes before completing the task
3. Fix any formatting, linting, build, or test failures reported by `cast ci`
4. If changes are made to a workspace project, ensure the workspace configuration supports CI builds (e.g., use `default-members` to exclude platform-specific members that require system dependencies)
5. Verify all tests pass with `cast test` (automatically runs both `cargo test` and `npm test` as appropriate)

### Cast CI Behavior
- For **Rust projects** (with Cargo.toml): Runs `cargo fmt`, `clippy`, `build`, and `test`
- For **TypeScript/Node.js projects** (with package.json): Runs `npm install`, `npm run lint`, `npm run compile`, and `npm test`
- For **hybrid projects** (both Cargo.toml and package.json, e.g., Dioxus web apps): Runs both Rust and TypeScript CI checks
- Playwright tests are automatically run via `npm test` when a package.json with test script exists

### Cast Test Behavior
- For **Rust projects** (with Cargo.toml): Runs `cargo test`
- For **Node.js projects** (with package.json and test script): Runs `npm test`
- For **hybrid projects** (both Cargo.toml and package.json): Runs both `cargo test` and `npm test`
- Automatically detects project type and runs appropriate tests

## Toolchain Requirements by Framework

Different frameworks require different tooling beyond Rust:

### Dioxus Framework
Projects with `framework = "dioxus"` require:
- **Rust toolchain**: Always required (rustc, cargo, rustfmt, clippy)
- **Dioxus CLI (`dx`)**: Required for serving, building, and bundling Dioxus apps
  - Install via: `cargo install dioxus-cli --version 0.7.2`
  - Used for: `dx serve`, `dx build`, `dx bundle`
- **Node.js and npm**: Required for web platform projects
  - Used for: Installing and running Playwright tests, managing web dependencies
- **Playwright**: Required for end-to-end testing of web apps
  - Install via: `npm ci` (installs from package.json), then `npx playwright install --with-deps chromium`

### Cloudflare Pages Framework
Projects with `framework = "cloudflare-pages"` require:
- **Rust toolchain**: Always required
- **Wrangler CLI**: Required for deploying to Cloudflare
  - Install via: `npm install -g wrangler` or `cargo install wrangler`
- **Node.js and npm**: Typically required for Wrangler and other tools

### Rust Library and Binary Projects
Projects without a framework designation (pure Rust libraries and binaries) require:
- **Rust toolchain**: Only requirement (rustc, cargo, rustfmt, clippy)
  - Install via: Official rustup installer or GitHub Actions `actions-rust-lang/setup-rust-toolchain`
  - Components needed: rustfmt, clippy
- **No additional tools required**: Pure Rust projects only need the Rust toolchain

**Project identification:**
- Libraries: Projects with `src/lib.rs`, or `[lib]` section in Cargo.toml
- Binaries: Projects with `src/main.rs`, files in `src/bin/` directory, or `[[bin]]` sections in Cargo.toml
- Note: Projects can have both library and binary targets. Toolchain requirements remain the same regardless.

### Cast Install Command
The `cast install` command automates installation of framework-specific tooling:

```bash
# Install all required tools for a project
cast install

# Check which tools are installed
cast install check

# List required tools for current project
cast install list
```

**Using in GitHub Workflows:**
- GitHub workflows should only install Rust via `actions-rust-lang/setup-rust-toolchain`
- Use `cast install` for all other framework-specific tools (Dioxus CLI, Playwright, etc.)
- See `standards/docs/workflow-conventions.md` for workflow patterns and best practices (also available in `.github/WORKFLOW_CONVENTIONS.md`)
- See `standards/docs/toolchain.md` for complete toolchain management documentation

## Cast Workspace Structure

The Cast tooling has been restructured into a Cargo workspace at `cast/` to better organize the related projects:

### Workspace Layout
```
cast/
├── Cargo.toml          # Workspace configuration
├── Cast.toml           # Workspace Cast configuration
├── README.md           # Workspace documentation
├── ISSUES.md           # Workspace development tasks
├── core/               # cast_core library (previously "cast")
│   ├── src/            # Core functionality
│   ├── tests/          # Integration tests
│   ├── benches/        # Performance benchmarks
│   └── examples/       # Usage examples
├── cast_cli/           # cast_cli binary
│   └── src/            # CLI implementation
└── vscode_ext/         # VSCode extension (previously "cast_vscode")
    └── src/            # Extension TypeScript code
```

### Key Points
- The **core library** was renamed from `cast` to `cast_core` to avoid naming conflicts
- Use `cast_core` as the crate name in dependencies: `cast_core = { path = "../cast/core" }`
- The **CLI binary** is still named `cast` (the executable name) but the crate is `cast_cli`
- Build the CLI from the workspace: `cd cast && cargo build --release -p cast_cli`
- The workspace uses shared dependencies and version configuration
- When referencing Cast in documentation or code, use the workspace paths

### Building and Testing
```bash
# Build all workspace members
cd cast
cargo build --workspace

# Run tests on all workspace members
cargo test --workspace

# Run CI on specific member
cd cast/core && cast ci
cd cast/cast_cli && cast ci
```

### GitHub Workflows
Workflows reference the new workspace structure:
- Build path: `cast/cast_cli/` (directory name matches package name `cast_cli`)
- Binary path: `cast/target/release/cast` instead of `cast_cli/target/release/cast`
- See `.github/workflows/cast-ci.yml` and `.github/workflows/cast-cd.yml` for examples
