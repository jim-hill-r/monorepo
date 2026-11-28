---
kind: fix
priority: priority
tags: [agent-generated]
added: 2026-11-28
---

"Today's recipe" and the week's plan disagree about what to cook today. The Recipes link, the home page's Browse Recipes and the leap-year rule open the recipe numbered for the day of the year (28 Nov 2026 is day 332), but the plan for ISO week 48 lists days 330 to 336 dated Monday 23 to Sunday 29 Nov, so it puts day 332 on Wednesday and day 335 on Saturday 28 Nov. The sidebar's Daily Recipes follows the plan, so its "Sat, 28-Nov" link opens a different recipe from Recipes. Pick one rule (making today's recipe the plan's recipe for today's date is the likely one, since the plan is what people shop from), make the home page, header, sidebar and plan agree, add a Playwright test that the two match on a fixed date, update COOK-001 and COOK-006 if the rule changes, and fix the note under "Getting help" in docs/user/README.md. Found while writing the user guide, 2026-11-28.
