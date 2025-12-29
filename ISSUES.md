
# Priority Issues

- TODO (agent-generated): Cast Toolchain Command - See `cast/docs/toolchain-command-design.md` for complete design specification. Implementation broken into sequential phases below.

- DONE (agent-generated): Create a new `toolchain.rs` module in cast/src with basic structure, error types, and Tool enum
- DONE (agent-generated): Implement toolchain detection logic that reads Cast config and determines required tools based on framework
- DONE (agent-generated): Implement tool detection logic for Node.js, npm, Playwright, dx, wrangler (checking if installed and getting versions)
- DONE (agent-generated): Add unit tests for toolchain detection logic

- DONE (agent-generated): Implement `cast toolchain install` subcommand with options: --tool, --skip, --dry-run, --force
- DONE (agent-generated): Implement Node.js detection and provide installation guidance for user's platform (apt, brew, winget)
- DONE (agent-generated): Implement Dioxus CLI installation via `cargo install dioxus-cli --version 0.7.2`
- DONE (agent-generated): Implement Wrangler CLI installation via `npm install -g wrangler` (with cargo fallback)
- DONE (agent-generated): Implement Playwright installation via `npm ci` and `npx playwright install --with-deps chromium`
- DONE (agent-generated): Add progress output and proper error handling with helpful messages
- DONE (agent-generated): Add comprehensive tests for each tool installation method
- DONE (agent-generated): Update cast/README.md with cast toolchain install documentation and examples

- DONE (agent-generated): Implement `cast toolchain check` to verify all required tools are installed (with --verbose and --json options)
- DONE (agent-generated): Implement `cast toolchain list` to show installed tools and versions (with --required-only and --all options)
- DONE (agent-generated): Add tests for list subcommand

- DONE (agent-generated): Test toolchain command on Linux (GitHub Actions runner environment)
- TODO (agent-generated): Add macOS-specific installation paths and methods if different from Linux
- TODO (agent-generated): Document Windows-specific installation considerations in design doc

- DONE (agent-generated): Create a standards document for toolchain management in standards/docs/toolchain.md

- DONE (agent-generated): Update .github/workflows/cast-ci.yml to use `cast toolchain install` instead of manual tool installation
- DONE (agent-generated): Update .github/WORKFLOW_CONVENTIONS.md to document that workflows should only install Rust, all other tools via cast
- DONE (agent-generated): Add guidance to .github/copilot-instructions.md about using `cast toolchain` in workflows
- TODO (agent-generated): Test the updated workflow on a test PR to ensure it works correctly

# Backlog

# Priority Projects
- cast
- cast_cli
- cookbook
- cookbook/web
- cookbook/cloudflare
- cahokia
- pane
- pane-cloudflare

