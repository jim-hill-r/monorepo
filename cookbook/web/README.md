# web

The `web` project is the browser-based frontend for the Cookbook application. It is named "web" because it is the web platform target within the Cookbook workspace, delivering daily recipes and weekly meal plans as a WebAssembly application built with Dioxus and Rust.

## Features

- **Daily Recipes**: Browse recipes for each day of the year (1-365) at `/recipe/{day}`
  - Recipes are loaded from markdown files in the `../content` directory
  - Each recipe displays title, description, ingredients, instructions, prep/cook time, servings, and tags
  - Previous / next day links move between recipes, wrapping from day 365 back to day 1
- **Weekly Meal Plans**: Access meal plans for each week of the year (1-52) at `/plan/{week}`, with a shopping list that sums each ingredient across the week's recipes and lists staples (oils, salt and pepper, garlic) apart
- **Input Validation**: Automatically validates day and week numbers to ensure they're within valid ranges
- **Phone layout**: below 768 px the header stays on one row and its Home / Recipes / Plans links move into the sidebar, which the menu button opens
- **Responsive Routing**: Uses Dioxus Router for seamless navigation between pages
- **Markdown Data Provider**: Uses the `cookbook-data-md` library to read recipes from markdown files
- **Security headers**: `public/_headers` makes Cloudflare Pages send a Content-Security-Policy (the `<meta>` CSP in `src/main.rs` plus `frame-ancestors 'none'`, which a `<meta>` CSP can't set), `Referrer-Policy: no-referrer`, `X-Content-Type-Options`, `X-Frame-Options` and HSTS on every page. `dx bundle` copies `public/` into the build. Change the CSP in both places: the tests in `src/main.rs` fail if they drift apart. The CSP allows no inline styles (`style-src 'self'`), so style with classes in the CSS files in `assets/`, never a `style` attribute or `<style>` element; `tests/csp.spec.ts` loads every page under the real header and fails on any violation.
- **Themes**: Uses the shared [design system](../../design_system/README.md), earthy by default. The sidebar's Appearance picker switches between the five themes, light or dark, and accessible mode, and remembers the choice in the browser. The CSS in `assets/styling/` uses only `--ds-` tokens, never its own colours. `tests/design-system.spec.ts` runs axe on the main pages in all 20 combinations.

## Prerequisites

Before running this project, you need to have:
- Rust toolchain installed (visit [rustup.rs](https://rustup.rs))
- Dioxus CLI (`dx`) installed
- [mise](https://mise.jdx.dev) for toolchain management

### Installing Required Tools

#### Automated Installation (Recommended)

Install the toolchain (Rust, the wasm target and a matching `dx`) with mise:

```bash
cd cookbook/web
mise install
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

- `/` - Home page: the intro from `../content/intro.md`, rendered as Markdown with `cookbook_core::markdown`, and navigation cards
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
├── Cargo.toml       # Rust dependencies and project metadata
├── mise.toml        # Tools and standard tasks (mise run ci, dev, ...)
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
