# Cast

Highly opinionated tooling for rust monorepos.

This project aims to make managing Rust monorepos simple via a simple CLI that can be run locally, on CI, and in cloud environments.

A cast is a [group of crabs](https://www.originaldiving.com/blog/our-favourite-collective-nouns-for-sea-creatures#:~:text=A%20group%20of%20crabs%20is,crabs%20dominating%20access%20to%20food.).

# Dependencies

- Rust
- Cargo

# Build

- Run `cargo build --release`

# Features

## Project Management

### Creating New Projects

Cast can create new projects from templates located in the `templates/` directory:

```rust
use cast::projects;

// Create a new project
projects::new("/path/to/monorepo", "my_project_name").unwrap();
```

This will:
1. Copy the `templates/base` directory to create the project structure
2. Copy the `templates/library` directory, overwriting any files from base
3. Remove empty `.gitignore` placeholder files used for tracking empty directories in git

The resulting project will have a complete structure ready for development with:
- `Cargo.toml` for Rust dependencies
- `Cast.toml` for Cast-specific configuration
- Standard directories: `src/`, `tests/`, `benches/`, `docs/`, etc.
