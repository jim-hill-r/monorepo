# cookbook_data_md

A markdown-based implementation of the recipe storage system for the cookbook application.

## Overview

This library provides `MarkdownRecipeStore`, which implements the `RecipeReader` and `RecipeWriter` traits from `cookbook-core`. It allows reading and writing recipe data from/to markdown files in a content directory.

## Features

- **Read recipes from markdown files**: Parses markdown files with a specific format to extract recipe data
- **Write recipes to markdown files**: Saves recipe data as formatted markdown files
- **Build-time data loading**: Recipes are loaded from the content directory at store initialization
- **Full trait implementation**: Implements both `RecipeReader` and `RecipeWriter` traits

## Markdown Format

The full recipe and plan format, with what each field means for the book, is in
[docs/content-format.md](../docs/content-format.md). Recipe markdown files follow this structure:

```markdown
# Recipe Title

Optional description text goes here.

UUID: 550e8400-e29b-41d4-a716-446655440000
ID: day-1
Status: draft
Prep Time: 10 minutes
Cook Time: 20 minutes
Servings: 4
Yield: 12 pancakes
Source: Grandma Hill
Tags: tag1, tag2, tag3

## Equipment

- griddle

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
- **UUID**: `UUID: <uuid-string>` format (RFC 4122 compliant UUID). Used as the primary identifier. A file whose `UUID:` line is missing or invalid is not loaded (see below).

### Optional Fields
- **ID**: `ID: <legacy-id>`, the recipe's legacy ID. Without it the ID is the file name without `.md`, which for `{uuid}.md` files is the UUID. `MarkdownRecipeStore` writes this line whenever the ID differs from the UUID, so an ID such as `day-1` survives a save and reload.
- **Status**: `Status: placeholder`, `draft`, `tested` or `book-ready`. Without the line a recipe is a `draft`; any other value fails the file, so it is not loaded. Every file in `content/` states one.
- **Description**: Text between title and first section heading, leaving out the `UUID:`, `ID:`, `Status:`, `Prep Time:`, `Cook Time:`, `Servings:`, `Yield:`, `Source:` and `Tags:` lines
- **Prep Time**: `Prep Time: X minutes` format
- **Cook Time**: `Cook Time: X minutes` format
- **Servings**: `Servings: X` format
- **Yield**: `Yield: <text>`, what the recipe makes, such as `24 cookies`
- **Source**: `Source: <text>`, who or where the recipe came from, credited in the book
- **Equipment**: List items under `## Equipment` section
- **Tags**: `Tags: tag1, tag2, tag3` format (comma-separated)
- **Ingredients**: List items under `## Ingredients` section
- **Instructions**: Numbered list under `## Instructions` section

### UUID Support

Recipes support UUIDs as unique identifiers:
- Every recipe file must have a valid `UUID:` line. A recipe never gets a new random UUID on load, because plans reference recipes by UUID.
- `MarkdownRecipeStore::new` skips any recipe or plan file that fails to parse, or that reuses another recipe's UUID or ID. It logs a warning for each and lists them in `load_errors()`.
- `get_by_id`, `exists` and `delete` accept either a UUID string or a legacy ID.
- `update` and `save` match a recipe by UUID, or else by legacy ID; in the second case the stored UUID is kept, so plans still resolve.
- `EmbeddedRecipeStore` (the web app's compiled-in recipes) keys recipes the same way: `get_by_id` and `exists` accept a UUID string or a legacy ID such as `day-1`, `get_by_uuid` is a map lookup, and a recipe that reuses another's UUID or ID is skipped with a warning.
- UUIDs allow recipes to be renamed and moved without breaking references

## Usage

```rust
use cookbook_core::{RecipeReader, RecipeWriter, Recipe};
use cookbook_data_md::MarkdownRecipeStore;
use uuid::Uuid;

// Create a store from a content directory
let mut store = MarkdownRecipeStore::new("./content")?;

// Read recipes by UUID (primary method)
let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")?;
let recipe = store.get_by_uuid(&uuid)?;

// Read recipes by legacy ID (for backward compatibility)
let recipe = store.get_by_id("day-1")?;

// Read all recipes
let all_recipes = store.get_all()?;

// Write recipes (UUID is automatically generated if not provided)
let new_recipe = Recipe::new("new-recipe".to_string(), "New Recipe".to_string());
store.create(new_recipe)?;
```

## File Organization

Recipes are stored in individual markdown files named with their UUIDs:

- `{uuid}.md` - Recipe file named with its unique UUID
- Example: `550e8400-e29b-41d4-a716-446655440000.md`

Recipes can be accessed using their UUID or legacy ID:

```rust
// Access by UUID (recommended)
let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")?;
let recipe = store.get_by_uuid(&uuid)?;

// Access by legacy ID (for backward compatibility)
let recipe = store.get_by_id("day-1")?;  // Works with legacy day-based IDs
```

The UUID-based file organization enables recipes to be rearranged and moved without breaking references in plans or other features.

## Special Files

The store automatically skips certain files:
- `intro.md` - Not treated as a recipe file

## Testing

Run the test suite:

```bash
cargo test
```

All tests use isolated temporary directories to avoid conflicts.
