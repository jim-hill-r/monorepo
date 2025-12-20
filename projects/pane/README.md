# Pane

A Dioxus web application built with Rust.

## Prerequisites

- Rust (latest stable version)
- Dioxus CLI (`dx`)

To install the Dioxus CLI:

```bash
cargo install dioxus-cli
```

## Running Locally in Dev Mode

To run the application in development mode with hot-reloading:

```bash
dx serve
```

This will:
- Build the application
- Start a local development server (usually at http://localhost:8080)
- Watch for file changes and automatically rebuild
- Enable hot-reloading in the browser

### Alternative: Run with cargo

You can also run the application using cargo directly:

```bash
cargo run
```

Note: This method won't include hot-reloading capabilities.

## Building

To build the project for production:

```bash
dx build --release
```

Or using cargo:

```bash
cargo build --release
```

To check the project without building:

```bash
cargo check
```

## Development Tips

- The dev server will automatically reload when you save changes to Rust files
- Check the browser console for any errors or warnings
- Hot-reloading works for both component logic and styling changes

## Status

This is a placeholder project with basic Dioxus setup.
