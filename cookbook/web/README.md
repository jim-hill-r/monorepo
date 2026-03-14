# web

The `web` project is the browser-based frontend for the Cookbook application. It is named "web" because it is the web platform target within the Cookbook workspace, delivering daily recipes and weekly meal plans as a WebAssembly application built with Dioxus and Rust.

## Features

- **Daily Recipes**: Browse recipes for each day of the year (1-365) at `/recipe/{day}`
  - Recipes are loaded from markdown files in the `../content` directory
  - Each recipe displays title, description, ingredients, instructions, prep/cook time, servings, and tags
- **Weekly Meal Plans**: Access meal plans for each week of the year (1-52) at `/plan/{week}`
- **Input Validation**: Automatically validates day and week numbers to ensure they're within valid ranges
- **Responsive Routing**: Uses Dioxus Router for seamless navigation between pages
- **Markdown Data Provider**: Uses the `cookbook-data-md` library to read recipes from markdown files

## Prerequisites

Before running this project, you need to have:
- Rust toolchain installed (visit [rustup.rs](https://rustup.rs))
- Dioxus CLI (`dx`) installed
- Cast CLI for toolchain management (included in this monorepo)

### Installing Required Tools

#### Automated Installation (Recommended)

Use the Cast install command to install all required dependencies:

```bash
cd cookbook/web
cast install
```

This will automatically install:
- Dioxus CLI (`dx`) version 0.7.2
- Playwright (for end-to-end testing)
- All npm dependencies

#### Manual Installation (Alternative)

Install the Dioxus CLI tool manually:

```bash
cargo install dioxus-cli --version 0.7.2
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

## Routes

The application provides the following routes:

- `/` - Home page with navigation information
- `/recipe/{day}` - Recipe for a specific day (1-365)
- `/plan/{week}` - Meal plan for a specific week (1-52)

Invalid day or week numbers will display a helpful error message.

## Building

This project is part of a Cargo workspace. To build for production from within the `web` directory, use the npm scripts:

```bash
# Development bundle
npm run bundle

# Production bundle (optimized)
npm run bundle:release
```

Alternatively, you can use the `dx` command directly with the `--package` flag:

```bash
# From the web directory
dx bundle --package web --platform web --release
```

Or run from the parent cookbook directory without the `--package` flag:

```bash
# From the cookbook directory
cd ..
dx bundle --platform web --release
```

The output will be in the `dist/` directory and ready for deployment.

To build the project using cargo directly:

```bash
cargo build
```

To check the project without building:

```bash
cargo check
```

## Testing

This project includes Playwright end-to-end tests. The tests automatically start and stop the development server, so you can simply run:

```bash
npm test
```

For more details on testing, including other test modes and manual setup options, see the [tests README](tests/README.md).

## Project Structure

```
web/
├── src/
│   └── main.rs      # Main application entry point with routing
├── tests/
│   ├── routing.spec.ts  # Playwright tests for routes
│   └── README.md        # Testing documentation
├── Cargo.toml       # Rust dependencies, project metadata, and Cast configuration
├── Dioxus.toml      # Dioxus build configuration
├── package.json     # npm package for Playwright tests
├── playwright.config.ts  # Playwright test configuration
└── README.md        # This file
```

## Dependencies

This project uses:
- Dioxus 0.7 for building web applications with Rust
- Dioxus Router for client-side routing
- Playwright for end-to-end testing
- `cookbook-core` for recipe data models and traits
- `cookbook-data-md` for reading recipes from markdown files (not directly used in web, but provides parsing logic)

## Implementation Details

The web application uses an embedded recipe store that includes all recipe markdown files at compile time. This is necessary because the application runs in the browser (WebAssembly) where filesystem access is not available.

The `EmbeddedRecipeStore` implementation:
1. Uses Rust's `include_str!` macro to embed all 365 recipe files at compile time
2. Parses the markdown content using the same parsing logic as `MarkdownRecipeStore`
3. Stores all parsed recipes in a `HashMap` for fast lookup
4. Implements the `RecipeReader` trait from `cookbook-core`

When a user navigates to a recipe page (e.g., `/recipe/1`), the application:
1. Gets the global singleton `EmbeddedRecipeStore` instance
2. Fetches the recipe for the requested day using `get_by_day(day)`
3. Displays the full recipe information including title, description, metadata, ingredients, instructions, and tags

The embedded approach has several advantages:
- No runtime file I/O needed
- Works in WASM/browser environment
- Instant recipe access (no network requests)
- All recipes are guaranteed to be available

## Status

This project now displays actual recipe content loaded from embedded markdown files. The recipes are generated using the `cookbook-recipe-gen` tool and stored in the `content` directory, then embedded into the binary at compile time.
