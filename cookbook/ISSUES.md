# Priority Issues

- TODO (agent-generated): Investigate and fix the 213 Playwright test failures in `cookbook/web`. The test infrastructure now works (tests can run and server starts), but many tests are failing on assertions. This may be due to application bugs, outdated test expectations, or browser-specific issues. Run tests with `npm test` in cookbook/web directory.
- TODO: The plan for a specific week should aggregate all the ingredients required for the week into a shopping list.
- TODO: The app is not responsive for smaller screen sizes. For example, on a mobile device the sidebar covers the whole screen and cannot be closed. On mobile it should be closed by default.
- FIX: The daily recipes sidebar has seven recipes but they don't match what is showing up in the Week 52 plan. Consider using the same logic for determining what recipes belong in a week. For example, 28-Dec has `Fresh carrots Salad with lamb` in the sidebar, but that recipe is the fifth recipe in Week 52 plan. Maybe also consider just using the plan directly to populate the daily recipes to guarantee they match.
- TODO: Use ISO_8601 calendar standard to determine what week numbers everywhere in the app. When determining the recipes to show in the daily recipes on the sidebar, first find what week it is with that standard, then find the plan for that week, then display the recipes from that plan. Note, that means we need 53 plans because some years have 53 weeks.
- TODO: Add plans to `cookbook/core` that stores the 7 recipes that are included in that week's plan.
- TODO: Store plans in the `content` directory similar to the recipes.
- TODO: Refactor recipes to include a UUID in the frontmatter of the markdown and also name the recipe files with the UUID. Ensure that the plans now reference the UUID for the recipe rather than the day number. The intent here is to allow future capability to rearrange what recipes belong to plans (ie decoupling a specific recipe from the day it is intended to be used on)

# Backlog

# Priority Projects
- web