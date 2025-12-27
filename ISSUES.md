
# Priority Issues

- TODO: Upgrade rust to version 1.89 in github workflows.
- TODO: Upgrade rust to version 1.89 for all projects

## Cast Toolchain Command Epic (agent-generated)
This epic breaks down the complex task of adding a `cast toolchain` command into manageable subtasks.

### Phase 1: Design and Planning (agent-generated)
- TODO (agent-generated): Document toolchain requirements for each framework type (dioxus, cloudflare-pages, rust-library, etc.) including versions and installation methods
- TODO (agent-generated): Design the command interface for `cast toolchain install` including options for installing specific tools vs all tools
- TODO (agent-generated): Research cross-platform tool installation methods (Linux, macOS, Windows) for Node.js, npm, Playwright, Dioxus CLI, and Wrangler

### Phase 2: Core Implementation (agent-generated)
- TODO (agent-generated): Add new `Toolchain` command variant to the Commands enum in args.rs
- TODO (agent-generated): Create a new `toolchain.rs` module in cast/src with basic structure and error types
- TODO (agent-generated): Implement toolchain detection logic that reads Cast config and determines required tools
- TODO (agent-generated): Add unit tests for toolchain detection logic

### Phase 3: Tool Installation Implementation (agent-generated)
- TODO (agent-generated): Implement Node.js installation detection and guidance (Node.js typically installed via system package manager)
- TODO (agent-generated): Implement npm package installation for Playwright (via `npm ci` or `npm install`)
- TODO (agent-generated): Implement Dioxus CLI installation (via `cargo install dioxus-cli`)
- TODO (agent-generated): Implement Wrangler CLI installation (via `npm install -g wrangler` or `cargo install wrangler`)
- TODO (agent-generated): Add version checking and upgrade logic for installed tools
- TODO (agent-generated): Add comprehensive tests for each tool installation method

### Phase 4: Cross-Platform Support (agent-generated)
- TODO (agent-generated): Test toolchain command on Linux (GitHub Actions runner environment)
- TODO (agent-generated): Add macOS-specific installation paths and methods if different
- TODO (agent-generated): Document Windows-specific installation considerations (may be out of scope)

### Phase 5: Integration and Documentation (agent-generated)
- TODO (agent-generated): Add `cast toolchain install` command documentation to cast/README.md
- TODO (agent-generated): Add toolchain requirements documentation to cast/docs
- TODO (agent-generated): Create a standards document for toolchain management in standards/docs/

### Phase 6: GitHub Workflow Integration (agent-generated)
- TODO (agent-generated): Update .github/workflows/cast-ci.yml to use `cast toolchain install` instead of manual tool installation
- TODO (agent-generated): Update .github/WORKFLOW_CONVENTIONS.md to document that workflows should only install Rust, all other tools via cast
- TODO (agent-generated): Add guidance to copilot-instructions about using `cast toolchain` in workflows
- TODO (agent-generated): Test the updated workflow on a test PR to ensure it works correctly

### Phase 7: Additional Toolchain Commands (agent-generated)
- TODO (agent-generated): Implement `cast toolchain list` to show installed tools and versions
- TODO (agent-generated): Implement `cast toolchain check` to verify all required tools are installed without installing them
- TODO (agent-generated): Add tests for list and check commands

# Backlog

# Priority Projects
- cast
- cookbook
- cookbook/web
- cookbook/cloudflare
- cahokia
- pane
- pane-cloudflare

