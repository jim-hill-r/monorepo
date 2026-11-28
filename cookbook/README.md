# Cookbook

[![Eval coverage](https://raw.githubusercontent.com/jimhillr/monorepo/evals-results/badges/cookbook.svg)](https://github.com/jimhillr/monorepo/blob/evals-results/verdicts.json)

A Dioxus workspace for building web applications with Rust.

The live site is <https://jimhillr-cookbook.pages.dev>. To learn how to use
it (find a recipe, follow a week's plan, use the shopping list on a phone),
read the [user guide](docs/user/README.md).

The content is the product and will become a book; where the project is
heading is in the [mission](docs/mission.md) and [roadmap](docs/roadmap.md).

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
- **Shopping list**: `shopping::build_shopping_list` sums a week's ingredients per item and unit and lists staples apart
- **Calendar**: `calendar` says which recipe falls on which date from the weekly plans (today's recipe, a week's dated recipes and shopping list, the sidebar's weeks), taking any `PlanReader` and `RecipeReader`
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

### Historical note

The project was originally organized with day-based filenames (`day-1.md` through `day-365.md`) and has since moved to UUID-based names. The one-off tools that did that migration were removed once it was done; they are in the git history if a future migration needs a starting point.

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

Or use mise from the `web` directory:

```bash
mise run dev
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

## Release size

The workspace `[profile.release]` in `Cargo.toml` tunes the release wasm for
size (`opt-level = "z"`, `lto`, `codegen-units = 1`). `dx bundle --release`
builds with it through dx's `wasm-release` profile, which inherits from
`release`. `strip` is `"debuginfo"`, not `true`: stripping symbols also drops
the `target_features` section, and wasm-bindgen then refuses the wasm.
`core/tests/release_profile.rs` checks both.

Measured on 2026-11-27 with `dx` 0.7.10, as `web_bg.wasm` after wasm-bindgen
and before wasm-opt (the sandbox couldn't run wasm-opt; it shrinks both
further, and the 2026-11-25 deep review measured 3,432,336 bytes after it):

| Build | Bytes | Gzip |
| --- | --- | --- |
| No release profile | 3,621,099 | 959,911 |
| Size-tuned release profile | 1,918,087 | 615,229 |

That is 47% smaller (36% gzipped).

## Running Tests

To run all tests in the workspace:

```bash
cargo test
```
