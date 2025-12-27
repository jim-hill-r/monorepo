# Toolchain Management

This document defines standards for managing development toolchains across the monorepo.

## Overview

Projects in this monorepo use various frameworks that require specific development tools beyond the base Rust toolchain. This document outlines the toolchain requirements and management strategy.

## Base Toolchain

All projects require:
- **Rust toolchain** (rustc, cargo, rustfmt, clippy)
- Managed via `rustup` or GitHub Actions' `actions-rust-lang/setup-rust-toolchain`
- Current target version: 1.82 (stable)
- Planned upgrade: 1.89

## Framework-Specific Toolchains

### Dioxus Framework

Projects using Dioxus require framework specification in their configuration:

**Cast.toml example:**
```toml
framework = "dioxus"
```

**Cargo.toml metadata example:**
```toml
[package.metadata.cast]
framework = "dioxus"
```

#### Required Tools
1. **Dioxus CLI (`dx`)**
   - Version: 0.7.2 (current standard)
   - Installation: `cargo install dioxus-cli --version 0.7.2`
   - Purpose: Development server, building, and bundling
   - Commands: `dx serve`, `dx build`, `dx bundle`

2. **Node.js and npm** (for web platform)
   - Version: 20.x (LTS)
   - Installation: System package manager or nvm
   - Purpose: JavaScript tooling, dependency management
   - Required for: Playwright testing, web asset management

3. **Playwright** (for testing)
   - Installation: `npm ci` (from package.json), then `npx playwright install --with-deps chromium`
   - Purpose: End-to-end testing of web applications
   - Browser: Chromium (with system dependencies)

#### Optional Tools
- **TypeScript compiler**: For projects with TypeScript code (installed via npm)

### Cloudflare Pages Framework

Projects deploying to Cloudflare Pages (`framework = "cloudflare-pages"`) require:

#### Required Tools
1. **Wrangler CLI**
   - Installation options:
     - Via npm: `npm install -g wrangler`
     - Via cargo: `cargo install wrangler`
   - Purpose: Cloudflare Workers and Pages deployment
   - Configuration: wrangler.toml in project root

2. **Node.js and npm**
   - Version: 20.x (LTS)
   - Purpose: Wrangler and other JavaScript tools

### Rust Library/Binary

Projects using standard Rust tooling only require the base Rust toolchain.

## Toolchain Management Strategy

### Current State (Manual Installation)

Currently, toolchains are managed manually:
- **Local development**: Developers install tools as needed based on project README
- **CI/CD**: GitHub Actions workflows manually install required tools using action steps

Example workflow steps from `.github/workflows/cast-ci.yml`:
```yaml
- name: Setup Node.js
  uses: actions/setup-node@v4
  
- name: Install Playwright browsers
  run: |
    # Shell script to find and install Playwright
    # (Note: This is GitHub Actions YAML, not a shell script in the repo)
    find . -name "package.json" ... | while ...; do
      npx playwright install --with-deps chromium
    done
    
- name: Install Dioxus CLI
  run: cargo install dioxus-cli --version 0.7.2
```

These installation steps will eventually be replaced by `cast toolchain install`.

### Future State (Cast Toolchain Command)

A `cast toolchain` command is planned to automate toolchain management:

```bash
# Install all required tools for a project
cast toolchain install

# Check which tools are installed
cast toolchain check

# List required tools for current project
cast toolchain list
```

The command will:
1. Read the project's Cast configuration (Cast.toml or Cargo.toml metadata)
2. Determine required tools based on framework and project type
3. Check for existing installations
4. Install missing tools or suggest installation for system-managed tools
5. Verify tool versions match requirements

### GitHub Workflow Standard (Future)

Once `cast toolchain` is implemented, workflows should:
1. **Only install Rust** via `actions-rust-lang/setup-rust-toolchain`
2. **Use `cast toolchain install`** for all framework-specific tools
3. Keep workflow files simple and declarative
4. Delegate tool installation logic to the Cast tool

## Tool Version Management

### Version Pinning
- Tools should be pinned to specific versions in documentation and installation commands
- Version requirements should be encoded in Cast's toolchain module
- Examples: Dioxus CLI 0.7.2, Node.js 20.x

### Version Updates
- Tool versions should be updated via PR with testing
- Update checklist:
  1. Update version in Cast toolchain module
  2. Update documentation (README files, this document)
  3. Update GitHub Actions caches (cache keys include versions)
  4. Test on representative projects
  5. Update copilot-instructions.md

## Installation Verification

When installing tools, verify installation with:
- Dioxus CLI: `dx --version`
- Node.js: `node --version`
- npm: `npm --version`
- Playwright: `npx playwright --version`
- Wrangler: `wrangler --version`

## Cross-Platform Considerations

### Linux (GitHub Actions, WSL)
- Primary CI/CD environment: Ubuntu latest
- Use `apt` for system dependencies (e.g., Playwright browser dependencies)
- Most tools install via cargo or npm without issues

### macOS
- Use `brew` for system dependencies
- cargo and npm tools work identically to Linux

### Windows
- Not currently a priority platform
- Some tools may require different installation methods
- Document when implementing cross-platform support

## Best Practices

1. **Document tool requirements** in project README files
2. **Pin tool versions** to ensure reproducibility
3. **Cache installed tools** in CI/CD (GitHub Actions cache)
4. **Verify installations** before use
5. **Keep toolchain management centralized** in the Cast tool
6. **Update tools deliberately** with testing, not automatically

## Related Documentation

- Cast README: `/cast/README.md`
- Cast CI Workflow: `.github/workflows/cast-ci.yml`
- Workflow Conventions: `.github/WORKFLOW_CONVENTIONS.md`
- Copilot Instructions: `.github/copilot-instructions.md`
- Cast Toolchain Epic: `/ISSUES.md` (implementation plan)
