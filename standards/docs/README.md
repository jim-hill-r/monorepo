# Standards Documentation

This directory contains development standards and guidelines for the monorepo.

## Quick Links

- [Code quality](./code-quality.md) - The QUAL rules for design, code, APIs and each language, and the ratchet that keeps them green
- [Configuration](./configuration.md) - Configuration file standards
- [Decisions](./decisions/README.md) - Repo-wide decision records
- [Documentation](./documentation.md) - Documentation standards
- [Cost](./cost.md) - Each project's cost analysis (`docs/cost.toml`), the cost check and how an expense is approved
- [Evals](./evals.md) - Product requirements (`docs/requirements.md`) and the evals that check them (`evals.toml`)
- [Free Tiers](./free-tiers.md) - Free tier limits of every service we use, and the guards that keep us inside them
- [Git hooks](./git-hooks.md) - lefthook hooks that run the CI checks before every commit and push
- [Issue Management](./issue-management.md) - Issue tracking and TODO conventions
- [Runbooks](./runbooks/README.md) - Step-by-step runbooks for every task only a person can do
- [Naming](./naming.md) - Naming conventions
- [Plans](./plans/) - Repo-wide plans, such as [feedback on every change](./plans/2026-11-deploy-feedback.md), the [eval engine](./plans/2026-11-eval-engine.md), the [cost check](./plans/2026-11-cost-check.md), [cost tracking](./plans/2026-11-cost-tracking.md), the [threat check](./plans/2026-11-threat-check.md), the [design system](./plans/2026-11-design-system.md), [resource cleanup](./plans/2026-11-resource-cleanup.md), [code quality](./plans/2026-11-code-quality.md), [baked skills](./plans/2026-11-baked-skills.md) and [revenue](./plans/2026-11-revenue-402.md)
- [Pull Requests](./pull-requests.md) - How every PR is driven to merge or close
- [Repo health](./repo-health.md) - The README's health badges, and how PR triage checks them and fixes what is red
- [Scheduled tasks](./scheduled-tasks/) - The exact prompts of the Claude scheduled tasks, such as [PR triage](./scheduled-tasks/pr-triage.md) and the hourly [credit window check](./scheduled-tasks/credit-window.md)
- [Resources](./resources.md) - Every live resource is claimed, and a renamed or removed one is retired and deleted safely
- [Rust](./rust.md) - Rust coding standards
- [Security](./security.md) - Security rules and the checks that enforce them (secret scanning, ignored secret files, Dependabot coverage)
- [Testing](./testing.md) - Testing standards and requirements
- [Toolchain](./toolchain.md) - Toolchain management and requirements
- [Tech radar](./tech-radar.md) - Every approved, evaluated and held technology and vendor, with exemplar projects
- [TypeScript](./typescript.md) - TypeScript coding standards

---

# Languages

All code in this repository must be written in Rust unless there is a specific exception.

Exceptions:
- TypeScript (for vscode extensions only)
- Configuration files (TOML, YAML, JSON, etc.)
- Documentation (Markdown, etc.)

No shell scripts (bash, sh, etc.) should be used. All scripting and automation must be done in Rust.

For detailed Rust coding standards, see [rust.md](./rust.md).

# Testing

For unit: Rust normal testing (cargo test)
For integration: TBD (Leaning toward appium)

# Benchmarking

For profiling: samply
For reporting/regression: criterion

# Cross compilation

Always use: cross

# Error Handling

For applications: [anyhow](https://docs.rs/anyhow/latest/anyhow/index.html).  
For libraries: [thiserror](https://docs.rs/thiserror/latest/thiserror/index.html).

For future consideration: [RFC 9457](https://www.rfc-editor.org/rfc/rfc9457.html)

# Logging

Always use: tracing

# Parallelism

For compute-bound: Rayon.  
For IO-bound: async/await (tokio runtime)

# Gaming

For engine: Bevy

# Web applications

For framework: Dioxus
For http: Axum
For OpenAPI: Utopia
For reverse proxy: Pingora

# CIAM

Always use: Auth0

# Databases

For general purpose: committed files first, then committed SQLite read with rusqlite, then Cloudflare D1 when a deployed service must write at runtime ([decision 0004](./decisions/0004-database-engine.md)). SurrealDB is on hold (see the [tech radar](./tech-radar.md)).
For realtime stateful sync: SpacetimeDB

# CLIs

For traditional: Clap.  
For TUIs: Ratatui

# Math

For linear algebra: nalgebra
For matrices/arrays: ndarray
