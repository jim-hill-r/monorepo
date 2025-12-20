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

Cast can create new projects from exemplar projects. Exemplar projects are marked with `exemplar = true` in their `Cast.toml` file.

```rust
use cast::projects;

// Create a new project
projects::new("/path/to/monorepo", "my_project_name").unwrap();
```

This will:
1. Search for all exemplar projects in the `projects/` directory
2. Copy each exemplar project to the new project location (later exemplars overwrite earlier ones)
3. Remove empty `.gitignore` placeholder files used for tracking empty directories in git

The resulting project will have a complete structure ready for development with:
- `Cargo.toml` for Rust dependencies
- `Cast.toml` for Cast-specific configuration
- Standard directories: `src/`, `tests/`, `benches/`, `docs/`, etc.

To create your own exemplar projects, simply add `exemplar = true` to the project's `Cast.toml` file.

### Finding Projects with Changes

Cast can find projects with changes between two git refs. This is useful for CI/CD workflows to determine which projects need to be tested or built.

```rust
use cast::projects;

// Find projects with changes between two commits
let changed_projects = projects::with_changes(
    "/path/to/monorepo",
    "origin/main",  // base ref
    "HEAD"          // head ref
).unwrap();

for project in changed_projects {
    println!("Changed project: {}", project.display());
}
```

This will:
1. Get all changed files between the two git refs using `git diff`
2. Walk up the directory tree from each changed file to find the closest `Cast.toml`
3. Return a sorted, deduplicated list of project directories

The CLI command is available as:
```bash
cast project with-changes --base <base-ref> --head <head-ref>
```

This is used in CI workflows to efficiently run tests only on changed projects.

## Configuration

Cast uses a `Cast.toml` file to configure project-specific settings.

### Cast.toml Configuration Options

```toml
# Whether this project is an exemplar project (example/template)
# Optional: defaults to None/false if not specified
exemplar = true

# Whether this project is a proof of concept project
# Optional: defaults to None/false if not specified
proof_of_concept = true

# The type of project (e.g., "dioxus", "cloudflare-pages", "rust-library")
# Optional: defaults to None if not specified
project_type = "dioxus"
```

You can load and parse Cast.toml configuration in your code:

```rust
use cast::config::CastConfig;

// Load configuration from a Cast.toml file
let config = CastConfig::load("path/to/Cast.toml").unwrap();

// Check if project is an exemplar
if config.exemplar == Some(true) {
    println!("This is an exemplar project");
}

// Check if project is a proof of concept
if config.proof_of_concept == Some(true) {
    println!("This is a proof of concept project");
}

// Check project type
if let Some(project_type) = config.project_type {
    println!("Project type: {}", project_type);
}
```
