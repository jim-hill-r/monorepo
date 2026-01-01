# Cookbook Data MD

A markdown-based implementation of the recipe storage system for the cookbook application.

## Overview

This library provides `MarkdownRecipeStore`, which implements the `RecipeReader` and `RecipeWriter` traits from `cookbook-core`. It allows reading and writing recipe data from/to markdown files in a content directory.

## Features

- **Read recipes from markdown files**: Parses markdown files with a specific format to extract recipe data
- **Write recipes to markdown files**: Saves recipe data as formatted markdown files
- **Build-time data loading**: Recipes are loaded from the content directory at store initialization
- **Full trait implementation**: Implements both `RecipeReader` and `RecipeWriter` traits

## Markdown Format

Recipe markdown files follow this structure:

```markdown
# Recipe Title

Optional description text goes here.

UUID: 550e8400-e29b-41d4-a716-446655440000
Prep Time: 10 minutes
Cook Time: 20 minutes
Servings: 4
Tags: tag1, tag2, tag3

## Ingredients

- Ingredient 1
- Ingredient 2
- Ingredient 3

## Instructions

1. First step
2. Second step
3. Third step
```

### Required Fields
- **Title**: First `# ` heading in the file
- **ID**: Derived from the filename (without `.md` extension)

### Optional Fields
- **UUID**: `UUID: <uuid-string>` format (RFC 4122 compliant UUID). If not provided, a new UUID is automatically generated when the recipe is loaded.
- **Description**: Text between title and first section heading
- **Prep Time**: `Prep Time: X minutes` format
- **Cook Time**: `Cook Time: X minutes` format
- **Servings**: `Servings: X` format
- **Tags**: `Tags: tag1, tag2, tag3` format (comma-separated)
- **Ingredients**: List items under `## Ingredients` section
- **Instructions**: Numbered list under `## Instructions` section

### UUID Support

Recipes support UUIDs as unique identifiers:
- If a recipe file contains a valid `UUID:` field in the frontmatter, that UUID will be used
- If no UUID is present or the UUID is invalid, a new UUID is automatically generated
- This provides backward compatibility with existing recipes while enabling UUID-based features
- UUIDs allow recipes to be renamed and moved without breaking references

## Usage

```rust
use cookbook_core::{RecipeReader, RecipeWriter, Recipe};
use cookbook_data_md::MarkdownRecipeStore;
use uuid::Uuid;

// Create a store from a content directory
let mut store = MarkdownRecipeStore::new("./content")?;

// Read recipes
let recipe = store.get_by_id("carbonara")?;
let day_recipe = store.get_by_day(1)?;
let all_recipes = store.get_all()?;

// Read recipe by UUID
let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")?;
let recipe = store.get_by_uuid(&uuid)?;

// Write recipes
let new_recipe = Recipe::new("new-recipe".to_string(), "New Recipe".to_string());
store.create(new_recipe)?;
```

## Day-Based Recipes

Recipes can be organized by day of the year. Use the filename format `day-{N}.md` where N is between 1 and 366:

- `day-1.md` - Recipe for January 1st
- `day-100.md` - Recipe for day 100
- `day-365.md` - Recipe for December 31st

These can be accessed using `get_by_day(day)`:

```rust
let recipe = store.get_by_day(1)?;  // Gets recipe from day-1.md
```

## Special Files

The store automatically skips certain files:
- `intro.md` - Not treated as a recipe file

## Testing

Run the test suite:

```bash
cargo test
```

All tests use isolated temporary directories to avoid conflicts.
