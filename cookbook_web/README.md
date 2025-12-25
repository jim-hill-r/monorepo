# Cookbook

A Dioxus web application built with Rust.

## Prerequisites

Before running this project, you need to have:
- Rust toolchain installed (visit [rustup.rs](https://rustup.rs))
- Dioxus CLI (`dx`) installed

### Installing Dioxus CLI

Install the Dioxus CLI tool:

```bash
cargo install dioxus-cli
```

## Running in Development Mode

To run the application locally in development mode with hot-reload:

```bash
dx serve
```

This will:
- Start a local development server (typically at `http://localhost:8080`)
- Enable hot-reload, so changes to your code will automatically refresh the browser
- Provide detailed error messages and debugging information

### Development Server Options

You can customize the development server behavior:

```bash
# Serve on a specific port
dx serve --port 3000

# Open the browser automatically
dx serve --open

# Enable verbose logging
dx serve --verbose
```

## Building

To build the project for production:

```bash
dx bundle
```

The output will be in the `dist/` directory and ready for deployment.

**Note:** The `Dioxus.toml` configuration disables wasm-opt (by setting level to 0) to avoid SIGABRT errors during release builds. This is a known issue where wasm-opt can crash on certain builds.

To build the project using cargo directly:

```bash
cargo build
```

To check the project without building:

```bash
cargo check
```

## Project Structure

```
cookbook_web/
├── src/
│   └── main.rs      # Main application entry point
├── Cargo.toml       # Rust dependencies and project metadata
├── Dioxus.toml      # Dioxus build configuration
├── Cast.toml        # Cast project configuration
└── README.md        # This file
```

## Dependencies

This project uses Dioxus 0.7 for building web applications with Rust.

## Status

This is a placeholder project with basic Dioxus setup.
