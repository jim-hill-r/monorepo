# Rust Standards

This document defines the coding standards and best practices for Rust projects in this monorepo.

## Clippy Lints

All Rust projects must enforce strict clippy lints to prevent common errors and unsafe practices.

### Required Lints

The following lints must be configured in every Rust project's `Cargo.toml`. These are enforced by the standards audit tool with the following standard IDs:

- **LIN-001**: `unwrap_used = "warn"` and `expect_used = "warn"` in `[lints.clippy]`
- **LIN-002**: `unsafe_code = "forbid"` in `[lints.rust]`

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

#### Approved `unsafe` exceptions

A project that truly needs `unsafe` sets `unsafe_code = "deny"` instead of
`"forbid"` (an `allow` can't override `forbid`), puts
`#[allow(unsafe_code)]` on each block that needs it, and gives each block a
`// SAFETY:` comment saying why it is sound. Each exception is listed here
once Jim approves it.

| Project | Where | Why |
| --- | --- | --- |
| `my_lang` | `JitRunner::call` in `src/jit.rs` | Calling LLVM JIT-compiled machine code through a function pointer has no safe API. |

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

```toml
# workspace Cargo.toml
[workspace.lints.clippy]
unwrap_used = "warn"
expect_used = "warn"

[workspace.lints.rust]
unsafe_code = "forbid"
```

```toml
# member Cargo.toml
[lints]
workspace = true
```

## Enforcement

`mise run //:audit` checks LIN-001 and LIN-002 for every tracked Cargo.toml with a `[package]` (`standards/scripts/project-check.sh`), as errors that fail the audit. Packages in an on hold project (the `# On Hold Projects` list in `standards/docs/issues/README.md`) are exempt until it comes off hold. A stricter clippy level (`deny`, `forbid`) passes LIN-001. `unsafe_code = "deny"` passes LIN-002 only when a comment on the line directly above it says why the package needs its one `#[allow(unsafe_code)]` item, such as a platform entry point.

## Further reading

The sources behind these rules (the Rust API Guidelines, Clippy, Effective
Rust, Rust Design Patterns, and the general design books) are summarised,
with what we take from each, in
[athenaeum/resources/code_quality/](../../athenaeum/resources/code_quality/README.md).
