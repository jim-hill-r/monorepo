# Priority Issues

- TODO: Use ISO_8601 calendar standard to determine what week numbers everywhere in the app. When determining the recipes to show in the daily recipes on the sidebar, first find what week it is with that standard, then find the plan for that week, then display the recipes from that plan. Note, that means we need 53 plans because some years have 53 weeks.
- TODO: Add plans to `cookbook/core` that stores the 7 recipes that are included in that week's plan.
- TODO: Store plans in the `content` directory similar to the recipes.
- TODO: Refactor recipes to include a UUID in the frontmatter of the markdown and also name the recipe files with the UUID. Ensure that the plans now reference the UUID for the recipe rather than the day number. The intent here is to allow future capability to rearrange what recipes belong to plans (ie decoupling a specific recipe from the day it is intended to be used on)

# Backlog

# Priority Projects
- web