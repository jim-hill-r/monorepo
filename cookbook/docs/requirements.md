# Cookbook requirements

What the cookbook promises the people who cook from it. The format and the
rules are in [standards/docs/evals.md](../../standards/docs/evals.md); the
evals that check each requirement are in [evals.toml](../evals.toml).

Drafted by an agent on 2026-11-26 from the README, `content/intro.md`,
`ISSUES.md` and the goals slide of the
[2026-11-25 deep review](../../docs/reviews/2026-11-25-cookbook/deck.md).
Signed-in features (that deck's goal 6) are left out until Jim decides what
they are.

## COOK-001: There is a recipe for every day of the year

- Priority: must
- Why: The cookbook is one recipe a day, so today's recipe must always be
  there.
- Accept: days 1 to 365 each open a recipe with a title, ingredients and
  steps; the "today" links open today's recipe on every date, reusing day
  365 on 31 December of a leap year; a day outside 1 to 365 shows a clear
  error with a way home.

## COOK-002: Every week has a meal plan with a shopping list

- Priority: must
- Why: Jim plans and shops a week at a time.
- Accept: weeks 1 to 53 each open a plan with seven recipes dated Monday to
  Sunday of that ISO week, and a shopping list for those recipes; a week
  outside 1 to 53 shows a clear error with a way home.

## COOK-003: The shopping list adds up

- Priority: should
- Why: A list that drops repeated quantities sends Jim to the shop for the
  wrong amounts.
- Accept: each ingredient appears once with its quantities summed per unit,
  and staples (oil, salt and pepper, garlic) are listed separately.

## COOK-004: The cookbook is easy to use on a phone

- Priority: should
- Why: Jim cooks from his phone.
- Accept: at 375 px wide there is no sideways scrolling and the header fits
  on one row; the sidebar is hidden by default and opens and closes from the
  menu button.

## COOK-005: Every page is accessible

- Priority: must
- Why: Anyone, including people using a screen reader or keyboard, should be
  able to cook from it.
- Accept: the home, recipe, plan and not-found pages have no WCAG 2 A or AA
  violations found by axe, and the menu button reports whether the sidebar
  is open.

## COOK-006: It is easy to move between today, recipes and plans

- Priority: should
- Why: Most visits are "what's for dinner today" or "what's this week".
- Accept: every page has a header with Home, Recipes and Plans links, the
  home page links to today's recipe and this week's plan, and the sidebar
  lists this week's recipes and nearby plans.

## COOK-007: The recipes are real and varied

- Priority: should
- Why: The cookbook is Jim's; generated templates aren't worth cooking from.
- Accept: no two days share a recipe title, recipes span breakfast, lunch,
  dinner and dessert across many cuisines, and each one reads as a real dish
  a person could cook from its ingredients and steps.

## COOK-008: Each week has a theme and lists its staples

- Priority: could
- Why: The intro promises weekly themes and kitchen staples.
- Accept: each plan names its theme and lists the staples the week assumes
  are in the kitchen.

## COOK-009: Jim can rearrange which recipes are in a plan

- Priority: could
- Why: Plans should follow what Jim wants to cook, not a fixed year.
- Accept: Jim can move a recipe to another day or week, and links to the
  recipe still work afterwards.
