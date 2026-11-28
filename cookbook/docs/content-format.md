# Content format

The cookbook's content is the product and will become a book (Jim,
2026-11-28, [roadmap](roadmap.md)). This page is the format that content is
written in: one Markdown file per recipe and one per weekly plan, in
[`../content/`](../content/). The parser is `cookbook_data_md`
([`data_md/src/parse.rs`](../data_md/src/parse.rs)); the data model is
`Recipe` and `Plan` in `cookbook_core`.

Write by hand, or let the management app write the files for you once it
exists. Either way the files are the source of truth, reviewed as pull
requests like code.

## Recipe files

A recipe lives in `content/<uuid>.md`, named after its UUID.

```markdown
# Sourdough Sandwich Loaf

UUID: 550e8400-e29b-41d4-a716-446655440000
Status: tested

A soft, tangy loaf for the week's sandwiches.

Prep Time: 30 minutes
Cook Time: 45 minutes
Servings: 12
Yield: one 9-inch loaf
Source: Grandma Hill's recipe card, about 1972
Tags: bread, american, day-12

## Equipment

- 9-inch loaf pan
- Dutch oven

## Ingredients

- 500 g bread flour
- 350 g water

## Instructions

1. Mix the flour and water and rest for an hour.
2. Bake for 45 minutes.
```

### Fields

Field lines go between the title and the first `## ` section, one per line,
in any order. Anything else there is the description.

| Field | Required | Format | Meaning |
| --- | --- | --- | --- |
| Title | yes | the first `# ` heading | The recipe's name as printed. |
| `UUID:` | yes | an RFC 4122 UUID | The recipe's identity. Plans point at it, so it never changes. A file without a valid one is not loaded. |
| `Status:` | yes in `content/` | `placeholder`, `draft`, `tested` or `book-ready` | How far the recipe is on its way into the book (below). |
| `ID:` | no | text | A legacy ID such as `day-1`, kept while old links use it. |
| Description | no | paragraphs | The headnote: what the dish is and why it's here. |
| `Prep Time:` | no | `<n> minutes` | Hands-on time. |
| `Cook Time:` | no | `<n> minutes` | Time on the heat or in the oven. |
| `Servings:` | no | a whole number | How many people it feeds. |
| `Yield:` | no | text | What it makes, in words, when servings don't say it: `24 cookies`, `one 9-inch loaf`, `1 quart`. |
| `Source:` | no | text | Who or where the recipe came from, credited in the book. Name the person and, for a published recipe, the book or page and a link. |
| `Tags:` | no | comma-separated | Meal, cuisine and diet, plus the `day-<n>` tag of the day it holds. |

### Sections

| Section | Format | Meaning |
| --- | --- | --- |
| `## Equipment` | `- ` list | Tools beyond a basic kitchen: a stand mixer, a Dutch oven, a thermometer. Leave it out when a pan and a knife will do. |
| `## Ingredients` | `- ` list | One ingredient a line, quantity first (`2 cups flour`), so the shopping list can add them up. |
| `## Instructions` | `1. ` list | One step a line, in order. |

Any other `## ` section, such as `## Notes`, is kept in the file but not
read yet.

### Status

Every recipe file in `content/` states its status, so progress toward the
book is a count anyone can take from the files. The statuses, in order:

| Status | Meaning | Who sets it |
| --- | --- | --- |
| `placeholder` | Generated filler that holds a day's place until a real recipe replaces it. All 365 recipes started here on 2026-11-28. | The generator |
| `draft` | A real recipe written down, not yet cooked from this text. | A curator writing it |
| `tested` | A curator cooked it from this text and it worked. | A curator, after cooking it |
| `book-ready` | Tested, edited, with its source credited: ready to print. | A curator, at the final edit |

An unknown status fails the build
(`test_load_example_recipes` and
`test_every_recipe_file_states_a_known_status` in `data_md`), so a typo
never hides a recipe from the count. Outside `content/`, a file without a
`Status:` line is read as `draft`.

## Plan files

A plan lives in `content/week-<n>.md` and puts seven recipes on the days of
ISO week `<n>`, Monday to Sunday.

```markdown
# Week 1 Plan

Meal plan for ISO 8601 week 1.

Week: 1
Recipe UUIDs: <monday>, <tuesday>, <wednesday>, <thursday>, <friday>, <saturday>, <sunday>
```

| Field | Required | Meaning |
| --- | --- | --- |
| `Recipe UUIDs:` | yes | Exactly seven recipe UUIDs, comma-separated, Monday first. |
| `Week:` | no | The ISO week, which the file name already gives. |

The `Year:` and `Days:` lines and the `## Recipe Days` list in today's plan
files are left from the day-numbered layout and aren't read. The plan's
dates are always the current year's ISO week. The weekly theme and staples
(COOK-008) will be added as `Theme:` and `## Staples` by the
[next issue](../issues/plan-themes-and-staples.md).

## Other files

`content/intro.md` is the book's introduction, shown on the home page.
