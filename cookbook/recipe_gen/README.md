# Recipe Generation Tool

A command-line tool for generating 365 unique recipe markdown files for the Cookbook application, and migrating existing recipes to include UUIDs.

## Tools

### recipe-gen
Generates recipe markdown files with content.

### uuid-migration
Adds UUID frontmatter to existing recipe markdown files.

### filename-migration
Renames recipe files from day-based naming (day-X.md) to UUID-based naming ({uuid}.md).

### plan-uuid-migration
Updates plan files to reference recipe UUIDs instead of day numbers.

## Overview

This tool automatically generates recipe markdown files in the correct format for the cookbook content directory. It creates recipes across various categories (breakfast, lunch, dinner, dessert, etc.) and cuisines (Italian, Chinese, Mexican, Indian, etc.) to ensure variety throughout the year.

## Features

- **Generates 365 unique recipes**: One recipe for each day of the year (day-1.md through day-365.md)
- **Category diversity**: Breakfast, lunch, dinner, dessert, snack, appetizer, soup, salad, and beverage recipes
- **Cuisine variety**: 15 different cuisines including Italian, Chinese, Mexican, Indian, Japanese, French, Thai, and more
- **Proper markdown formatting**: All recipes follow the cookbook's markdown format specification
- **Intelligent tagging**: Automatically adds tags based on category, cuisine, cooking time, and dietary attributes
- **Validation**: Verifies all 365 day files exist after generation
- **UUID Migration**: Adds UUID frontmatter to existing recipe files to support the UUID-based architecture

## Usage

### Generating Recipes

From the `cookbook/recipe_gen` directory:

```bash
cargo run --bin recipe-gen
```

The tool will:
1. Generate a base set of hand-crafted recipe templates
2. Create additional recipe variations using intelligent combinations
3. Shuffle recipes for variety across days
4. Write each recipe to a markdown file (day-1.md through day-365.md)
5. Verify all files were created successfully

### Migrating Recipes to Include UUIDs

From the `cookbook/recipe_gen` directory:

```bash
cargo run --bin uuid-migration
```

The UUID migration tool will:
1. Scan the content directory for all day-*.md files
2. Check each file for existing UUID frontmatter
3. Generate a new UUID for files without one
4. Add the UUID field to the recipe markdown frontmatter
5. Report statistics on the migration

**Note**: The migration tool is idempotent - running it multiple times will not create duplicate UUIDs or modify files that already have UUIDs.

### Renaming Recipe Files to UUID-Based Names

After all recipes have UUIDs in their frontmatter, you can rename the files to use UUIDs:

```bash
cargo run --bin filename-migration
```

The filename migration tool will:
1. Scan the content directory for all day-*.md files
2. Extract the UUID from each file's frontmatter
3. Show a preview of which files will be renamed
4. Ask for confirmation before proceeding
5. Rename each file from `day-X.md` to `{uuid}.md`
6. Report statistics on the migration

**Important**: 
- The `uuid-migration` tool must be run FIRST to ensure all files have UUID frontmatter
- This operation cannot be easily undone - make sure you have a backup!
- After running this tool, you'll need to update build.rs and other code that references day-based filenames

### Updating Plan Files to Use UUIDs

After recipes have UUIDs, you can update plan files to reference UUIDs instead of day numbers:

```bash
cargo run --bin plan-uuid-migration
```

The plan UUID migration tool will:
1. Scan the content directory for all week-*.md files
2. Read the day numbers from each plan's `Days:` line
3. Look up the corresponding recipe UUIDs
4. Add a `Recipe UUIDs:` line to each plan file
5. Report statistics on the migration

**Benefits**:
- Plans reference recipes by UUID instead of day number
- Recipes can be rearranged without breaking plans
- Backward compatible - both `Days:` and `Recipe UUIDs:` lines are kept
- Parser tries UUID-based format first, falls back to day-based format

**Note**: This tool requires that all recipes have UUID frontmatter (run `uuid-migration` first).

## Recipe Format

Each generated recipe includes:

- **Title**: Descriptive recipe name
- **Description**: Brief description of the dish
- **Prep Time**: Time in minutes
- **Cook Time**: Time in minutes
- **Servings**: Number of servings
- **Tags**: Category, cuisine, and descriptive tags (e.g., "quick", "vegetarian")
- **Ingredients**: List of ingredients with quantities
- **Instructions**: Step-by-step cooking instructions

Example output:
```markdown
# Italian Chicken with Broccoli and Rice

A delicious italian dish combining chicken with fresh broccoli served with rice

Prep Time: 15 minutes
Cook Time: 25 minutes
Servings: 4
Tags: dinner, italian, day-42

## Ingredients

- 1 lb chicken
- 2 cups broccoli
- 1 cup rice
- 2 tablespoons olive oil
- Salt and pepper to taste
- 2 cloves garlic, minced

## Instructions

1. Prepare chicken by cutting into bite-sized pieces
2. Heat olive oil in a large pan
3. Cook chicken until golden
4. Add broccoli and garlic, sauté until tender
5. Meanwhile, cook rice according to package directions
6. Combine everything and season with salt and pepper
7. Serve hot and enjoy
```

## Running Tests

```bash
cargo test
```

## Technical Details

- **Base templates**: The tool includes hand-crafted recipe templates for quality and authenticity
- **Generated variations**: Additional recipes are generated using intelligent combinations of proteins, vegetables, grains, categories, and cuisines
- **Randomization**: Uses the `rand` crate to shuffle recipes and create variations
- **Storage**: Uses the `cookbook-data-md` library to write recipes in the correct format

## Regenerating Recipes

The tool will skip any recipe files that already exist. To regenerate all recipes:

1. Delete existing day-*.md files from the content directory (or back them up)
2. Run the tool again

## Recipe Quality

The generated recipes are:
- **Syntactically valid**: All follow the cookbook markdown format
- **Semantically reasonable**: Ingredient and instruction combinations make culinary sense
- **Diverse**: Wide variety of categories and cuisines
- **Tagged appropriately**: Automatic tagging for easy searching and filtering

For production use, recipes should be reviewed and enhanced with more specific quantities, techniques, and cooking tips.
