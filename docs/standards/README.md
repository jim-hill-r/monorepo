# Languages

- Rust
- TypeScript (for vscode extensions only)

# Testing
For unit: Rust normal testing (cargo test)

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

For framework: Dixous
For http: Axum
For OpenAPI: Utopia

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
