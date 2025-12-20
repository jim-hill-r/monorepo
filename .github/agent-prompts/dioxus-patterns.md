# Dioxus Project Patterns

## Overview
This monorepo contains multiple Dioxus-based web applications. This document captures the conventions and patterns used for Dioxus projects.

## Project Structure
Standard Dioxus projects in this monorepo follow this structure:
```
project_name/
├── Cargo.toml
├── README.md
├── .gitignore
├── src/
│   └── main.rs
└── tests/
    └── integration_tests.rs
```

## Cargo.toml Configuration
- **Edition**: Use Rust edition `2024`
- **Dioxus Version**: `0.6` with `web` feature enabled
- **Features**: Include `default = ["web"]` and `web = []`

Example:
```toml
[package]
name = "project_name"
version = "0.1.0"
edition = "2024"

[dependencies]
dioxus = { version = "0.6", features = ["web"] }

[features]
default = ["web"]
web = []
```

## Application Code Pattern
- Use `dioxus::prelude::*` for imports
- Create a `main()` function that calls `dioxus::launch(App)`
- Define an `App` component with `#[component]` attribute
- Use `rsx!` macro for defining UI elements
- Components should return `Element` type

## Styling
- Use inline styles via the `style` attribute in rsx!
- Follow a consistent color scheme (e.g., sky blue theme: `#0c4a6e`, `#0e7490`)
- Make components reusable with props when appropriate

## Testing
All Dioxus projects should include integration tests that verify:
1. Project structure (Cargo.toml, README.md, src/main.rs exist)
2. Cargo.toml has correct package name
3. Cargo.toml includes dioxus dependency
4. README describes the project purpose
5. main.rs imports dioxus prelude
6. main.rs has a main function with dioxus::launch
7. App component exists with #[component] attribute
8. .gitignore excludes build artifacts (/target)

## .gitignore
All Dioxus projects should exclude:
```
/target
/dist
Cargo.lock
.DS_Store
```

## Documentation
README.md should include:
- Project description
- Features/capabilities
- Technology stack (mention Dioxus and Rust edition)
- Build and run instructions
- Target audience (if applicable)
- License reference

## Related Projects
- `pane`: Simple placeholder Dioxus application
- `cahokia`: Multi-platform Dioxus workspace with core, desktop, mobile, ui, and web modules
- `blueeel`: Reading education application with educational theme and multiple sections

## Build and Test Commands
```bash
# Check compilation
cargo check

# Build the project
cargo build

# Run tests
cargo test

# Run the application
cargo run

# For web deployment with dx CLI
dx serve
```
