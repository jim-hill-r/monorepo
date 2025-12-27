# Languages

All code in this repository must be written in Rust unless there is a specific exception.

Exceptions:
- TypeScript (for vscode extensions only)
- Configuration files (TOML, YAML, JSON, etc.)
- Documentation (Markdown, etc.)

No shell scripts (bash, sh, etc.) should be used. All scripting and automation must be done in Rust.

For detailed Rust coding standards, see [rust.md](./rust.md).

# Development Standards

- [Configuration](./configuration.md) - Configuration file standards
- [Documentation](./documentation.md) - Documentation standards
- [Naming](./naming.md) - Naming conventions
- [Toolchain](./toolchain.md) - Toolchain management and requirements
- [TypeScript](./typescript.md) - TypeScript coding standards

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

# Web applicaitons

For framework: Dioxus
For http: Axum
For OpenAPI: Utopia
For reverse proxy: Pingora

# CIAM

Always use: Auth0

# Databases

For general purpose: SurrealDB
For realtime stateful sync: SpacetimeDB

# CLIs

For traditional: Clap.  
For TUIs: Ratatui

# Math

For linear algebra: nalgebra
For matrices/arrays: ndarray
