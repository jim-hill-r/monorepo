# Recipe Generation Tool

A command-line tool for generating 365 unique recipe markdown files for the Cookbook application.

## Overview

This tool automatically generates recipe markdown files in the correct format for the cookbook content directory. It creates recipes across various categories (breakfast, lunch, dinner, dessert, etc.) and cuisines (Italian, Chinese, Mexican, Indian, etc.) to ensure variety throughout the year.

## Features

- **Generates 365 unique recipes**: One recipe for each day of the year (day-1.md through day-365.md)
- **Category diversity**: Breakfast, lunch, dinner, dessert, snack, appetizer, soup, salad, and beverage recipes
- **Cuisine variety**: 15 different cuisines including Italian, Chinese, Mexican, Indian, Japanese, French, Thai, and more
- **Proper markdown formatting**: All recipes follow the cookbook's markdown format specification
- **Intelligent tagging**: Automatically adds tags based on category, cuisine, cooking time, and dietary attributes
- **Validation**: Verifies all 365 day files exist after generation

## Usage

From the `cookbook/recipe_gen` directory:

```bash
cargo run
```

The tool will:
1. Generate a base set of hand-crafted recipe templates
2. Create additional recipe variations using intelligent combinations
3. Shuffle recipes for variety across days
4. Write each recipe to a markdown file (day-1.md through day-365.md)
5. Verify all files were created successfully

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
