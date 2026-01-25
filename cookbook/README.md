# Cookbook

A Dioxus workspace for building web applications with Rust.

## Development

This workspace contains member crates for different platforms and features:

```
cookbook/
├─ cloudflare/
│  ├─ ... # Cloudflare Pages deployment configuration
├─ core/
│  ├─ ... # Core business logic and data models (library)
├─ data_md/
│  ├─ ... # Markdown-based recipe storage implementation (library)
├─ recipe_gen/
│  ├─ ... # Recipe generation tool for creating 365 recipes
├─ web/
│  ├─ ... # Web specific UI/logic
```

## Core Library

The `cookbook-core` library contains shared business logic and data models:
- **Recipe struct**: Data model for recipe information including title, ingredients, instructions, timing, and tags
- **RecipeReader trait**: Trait for reading recipes from a data source (by ID, by day, all recipes, by tag)
- **RecipeWriter trait**: Trait for writing recipes to a data source (create, update, delete, save/upsert)
- **RecipeError**: Error type for recipe operations with variants for NotFound, StorageError, InvalidData, and AlreadyExists
- Designed to be consumed by web, cloudflare, and future platform crates

## Data MD Library

The `cookbook-data-md` library implements the RecipeReader and RecipeWriter traits for markdown-based storage:
- **MarkdownRecipeStore**: Reads and writes recipes from/to markdown files in a content directory
- Parses markdown files with a specific format to extract recipe data (title, description, ingredients, instructions, metadata)
- Recipes are stored in UUID-based filenames (e.g., `550e8400-e29b-41d4-a716-446655440000.md`)
- Supports legacy day-based ID lookups for backward compatibility (e.g., `get_by_id("day-1")`)
- Data is loaded at build time from the content directory
- See [data_md/README.md](./data_md/README.md) for detailed markdown format and usage

## Recipe Generation Tool

The `cookbook-recipe-gen` tool generates 365 unique recipe markdown files for the cookbook:
- **Automated recipe generation**: Creates one recipe for each day of the year (day-1.md through day-365.md)
- **Category and cuisine diversity**: Includes breakfast, lunch, dinner, dessert, and more across 15 different cuisines
- **Proper formatting**: All generated recipes follow the cookbook's markdown format specification
- **Validation**: Includes tests to verify all 365 day files exist and are valid
- See [recipe_gen/README.md](./recipe_gen/README.md) for usage instructions and details

To generate recipes:
```bash
cd recipe_gen
cargo run
```

## Recipe UUID Architecture

The cookbook uses a UUID-based architecture where each recipe is identified by a unique UUID (Universally Unique Identifier). This enables recipes to be rearranged, renamed, and moved without breaking references in plans or other features.

### Current Architecture

- **Recipe Files**: Named with their UUID (e.g., `550e8400-e29b-41d4-a716-446655440000.md`)
- **Primary Identifier**: UUID field in recipe frontmatter
- **Legacy Support**: Day-based IDs (e.g., "day-1") are supported for backward compatibility
- **Plan References**: Plans reference recipes by UUID, enabling flexible recipe organization

### Historical Migration Documentation

The project was originally organized with day-based filenames (`day-1.md` through `day-365.md`). A migration to UUID-based architecture was completed to support more flexible recipe management. The documentation below is preserved for reference and for any future migrations.

### Migration Process Overview (Historical Reference)

The migration is performed using four tools in the `recipe_gen` directory, executed in order:

1. **uuid-migration**: Adds UUID frontmatter to all recipe files
2. **filename-migration**: Renames recipe files from `day-X.md` to `{uuid}.md`
3. **plan-uuid-migration**: Updates plan files to reference recipe UUIDs
4. **migration-validator**: Validates that the migration completed successfully

### Prerequisites

Before starting the migration:

- **Backup your content directory** - The migration modifies files and cannot be easily undone
- Ensure all recipe files exist and are valid markdown
- Ensure all plan files (week-*.md) exist and are properly formatted
- Close any editors or processes that may have recipe files open

### Step-by-Step Migration Guide

Navigate to the recipe generation tool directory:
```bash
cd cookbook/recipe_gen
```

#### Step 1: Add UUIDs to Recipe Files

Add UUID frontmatter to all existing recipe files:

```bash
cargo run --bin uuid-migration
```

This tool:
- Scans all `day-*.md` files
- Generates a UUID for each recipe
- Adds a `UUID:` field to each recipe's frontmatter
- Is idempotent (safe to run multiple times)

#### Step 2: Rename Files to UUID-Based Names

Rename recipe files to use their UUID as the filename:

```bash
cargo run --bin filename-migration
```

This tool:
- Shows a preview of files to be renamed
- Asks for confirmation before proceeding
- Renames each file from `day-X.md` to `{uuid}.md`

**⚠️ WARNING**: This operation cannot be easily undone. Make sure you have a backup!

#### Step 3: Update Plan Files

Update plan files to reference recipe UUIDs:

```bash
cargo run --bin plan-uuid-migration
```

This tool:
- Updates all `week-*.md` files
- Adds a `Recipe UUIDs:` field to each plan
- Maintains backward compatibility by keeping the original `Days:` field

#### Step 4: Validate the Migration

Verify that the migration completed successfully:

```bash
cargo run --bin migration-validator
```

This tool checks:
- All recipe files have valid UUID frontmatter
- All recipe files use UUID-based naming
- No orphaned `day-*.md` files remain
- All plan files have Recipe UUIDs
- All recipe UUIDs in plans reference existing files

The validator exits with code 0 on success or code 1 if issues are found.

### Post-Migration Steps

After completing the migration:

1. Update any build scripts that reference day-based filenames
2. Update data parsers to handle UUID-based file names
3. Update tests to use UUIDs instead of day-based IDs
4. Update the web application to use UUID-based recipe references
5. Test the application thoroughly to ensure all recipes load correctly

### Troubleshooting

- **Missing UUIDs**: Run `uuid-migration` again (it's idempotent)
- **Files not renamed**: Ensure step 1 completed successfully before running `filename-migration`
- **Plan migration fails**: Verify all recipes have UUIDs before updating plans
- **Validation fails**: Check the detailed error output and address issues individually

### Detailed Tool Documentation

For detailed information about each migration tool, including technical details, examples, and test coverage, see [recipe_gen/README.md](./recipe_gen/README.md).

## Deployment

The cookbook web application can be deployed to Cloudflare Pages using the `cloudflare` deployment project in this workspace. See the [cloudflare README](./cloudflare/README.md) for deployment instructions.

## Platform crates

Each platform crate contains the entry point for the platform, and any assets, components and dependencies that are specific to that platform.

### Serving Your App

Navigate to the platform crate of your choice:
```bash
cd web
```

and serve:

```bash
dx serve
```

Or use the Cast CLI from the workspace root:

```bash
cast run
```

## Building

To build the entire workspace:

```bash
cargo build
```

To check the workspace without building:

```bash
cargo check
```

## Running Tests

To run all tests in the workspace:

```bash
cargo test
```

## Code Coverage

The cookbook project uses [cargo-tarpaulin](https://github.com/xd009642/tarpaulin) for code coverage reporting. Tarpaulin analyzes which lines of code are executed during tests, helping identify untested code paths.

### Installing Tarpaulin

If you don't have tarpaulin installed, install it with:

```bash
cargo install cargo-tarpaulin
```

### Running Coverage Reports

To generate a code coverage report:

```bash
cd cookbook
cargo tarpaulin
```

This will:
- Run all tests in the workspace
- Generate an HTML report in `target/tarpaulin/index.html`
- Display a coverage summary in the terminal
- Fail if coverage drops below 80% (per [testing standards](../standards/docs/testing.md))

### Viewing the HTML Report

Open the generated HTML report in your browser to see detailed line-by-line coverage:

```bash
# Linux
xdg-open tarpaulin-report.html

# macOS
open tarpaulin-report.html

# Windows
start tarpaulin-report.html
```

The HTML report shows:
- Overall coverage percentage for each file
- Line-by-line highlighting of covered (green) and uncovered (red) code
- Coverage statistics per file and function

### Configuration

Coverage settings are configured in `tarpaulin.toml`:
- Excludes test files from coverage calculations
- Sets the 80% coverage threshold (from testing standards)
- Generates both HTML and terminal output
- Timeout set to 120 seconds for all tests

### Interpreting Results

Current coverage baseline: **~76%**

Focus on improving coverage for:
- Error handling paths and edge cases
- Business logic in core and data_md libraries
- UI rendering logic in the web application

See the [Testing Standards](../standards/docs/testing.md) for coverage goals:
- 80% coverage for critical business logic
- 100% coverage for public APIs and error handling
