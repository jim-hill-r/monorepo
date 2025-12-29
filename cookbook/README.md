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
- Supports day-based recipe organization (day-1.md through day-366.md)
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
