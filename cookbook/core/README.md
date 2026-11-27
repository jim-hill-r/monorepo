# cookbook_core

The core library for the Cookbook application — shared traits and types for recipe and plan data.

## Overview

This library provides the core data models and traits for the cookbook application, including `Recipe`, `Plan`, `RecipeReader`, `RecipeWriter`, `PlanReader`, and `PlanWriter`.

## UUID-Based Architecture

The cookbook uses a UUID-based architecture where each recipe is uniquely identified by a UUID. This enables:

- **Recipe Rearrangement**: Recipes can be moved between days, weeks, or plans without breaking references.
- **Flexibility**: Recipe files can be renamed or reorganized without affecting functionality.
- **Data Integrity**: Plans reference recipes by UUID, ensuring relationships remain valid.
- **Migration Support**: Legacy day-based IDs are supported for backward compatibility.

## Key Components

- `Recipe`: The main recipe data structure with UUID as primary identifier
- `RecipeReader`: Trait for reading recipes from a data source
- `RecipeWriter`: Trait for writing recipes to a data source
- `Plan`: Weekly meal plan that references recipes by UUID
- `PlanReader`: Trait for reading plans from a data source
- `PlanWriter`: Trait for writing plans to a data source
- `markdown`: A small Markdown parser (paragraphs, `#` headings, `**strong**` and `*emphasis*`) that turns prose such as `content/intro.md` into blocks the web app renders as elements, never as injected HTML
- `shopping`: Parses ingredient lines (`"1 1/2 cups flour"`) and sums a week's ingredients into one line per item, quantities added per unit, with staples (oils, salt and pepper, garlic) listed apart

## Usage

Add to your `Cargo.toml`:

```toml
cookbook_core = { path = "../core" }
```

Then use the traits in your implementation:

```rust
use cookbook_core::{RecipeReader, RecipeWriter, PlanReader, PlanWriter};
```
