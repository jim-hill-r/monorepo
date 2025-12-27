# Rust Standards

This document defines the coding standards and best practices for Rust projects in this monorepo.

## Clippy Lints

All Rust projects must enforce strict clippy lints to prevent common errors and unsafe practices.

### Required Lints

The following lints must be configured in every Rust project's `Cargo.toml`:

```toml
[lints.clippy]
unwrap_used = "warn"
expect_used = "warn"

[lints.rust]
unsafe_code = "forbid"
```

#### Rationale

- **`unwrap_used = "warn"`**: Warns about panic-inducing `.unwrap()` calls. Use proper error handling with `?` operator, `match`, or `if let` instead. Set to "warn" rather than "forbid" to allow compatibility with derive macros.
- **`expect_used = "warn"`**: Warns about panic-inducing `.expect()` calls. Use proper error handling patterns instead. Set to "warn" rather than "forbid" to allow compatibility with derive macros.
- **`unsafe_code = "forbid"`**: Forbids unsafe code blocks. All code must be memory-safe by default. If unsafe is truly necessary, it requires explicit discussion and approval. Note: This is a rustc lint, not a clippy lint.

### Proper Error Handling

Instead of using `.unwrap()` or `.expect()`:

**Bad:**
```rust
let value = some_option.unwrap();
let result = some_result.expect("Failed");
```

**Good:**
```rust
// For Option types
let value = some_option.ok_or_else(|| anyhow::anyhow!("Value not found"))?;

// For Result types
let result = some_result?;

// Or use match/if let when appropriate
match some_option {
    Some(value) => value,
    None => return Err(anyhow::anyhow!("Value not found")),
}
```

## Configuration

Add the lints section to your project's `Cargo.toml`:

```toml
[package]
name = "your-project"
version = "0.1.0"
edition = "2021"

[lints.clippy]
unwrap_used = "warn"
expect_used = "warn"

[lints.rust]
unsafe_code = "forbid"

[dependencies]
# ... your dependencies
```

For workspace projects, you can also define lints at the workspace level using `[workspace.lints]` and inherit them in member crates with `lints.workspace = true`.
