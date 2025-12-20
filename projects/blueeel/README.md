# Blue Eel Education

A Dioxus-based web application for an educational platform.

## Overview

Blue Eel Education is a modern educational platform built with Dioxus, providing a responsive and interactive learning experience. The application features course browsing, content delivery, and a user-friendly interface.

## Features

- **Responsive Design**: Works seamlessly across different devices
- **Course Catalog**: Browse and explore available courses
- **Interactive UI**: Built with modern web technologies using Dioxus
- **Modular Components**: Well-structured component architecture

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo
- Dioxus CLI (optional, for development)

### Installation

```bash
# Install Dioxus CLI (optional)
cargo install dioxus-cli

# Navigate to the project directory
cd projects/blueeel
```

### Building

```bash
# Check the project
cargo check

# Build for development
cargo build

# Build for release
cargo build --release
```

### Running

```bash
# Using Dioxus CLI (recommended for development)
dx serve

# Or using cargo
cargo run
```

The application will be available at `http://localhost:8080` (when using dx serve).

## Project Structure

```
blueeel/
├── src/
│   └── main.rs          # Main application entry point with components
├── Cargo.toml           # Project dependencies and configuration
├── .gitignore          # Git ignore rules
└── README.md           # This file
```

## Components

- **App**: Main application component that orchestrates the layout
- **Header**: Navigation header with site branding and menu
- **MainContent**: Primary content area featuring courses and information
- **CourseCard**: Reusable component for displaying course information
- **Footer**: Site footer with copyright information

## Development

This project follows standard Rust development practices:

1. Make changes to the code
2. Run `cargo check` to verify compilation
3. Test in the browser using `dx serve`
4. Build for production with `cargo build --release`

## Technologies

- **Dioxus**: Modern React-like framework for Rust
- **Rust**: System programming language for performance and safety
- **WASM**: WebAssembly target for browser execution

## License

See the repository root LICENSE.md file for details.
