# design_system requirements

What the design system promises the apps that use it and the people who
use those apps. The format and the rules are in
[standards/docs/evals.md](../../standards/docs/evals.md); the evals that
check each requirement are in [evals.toml](../evals.toml).

Drafted by an agent on 2026-11-27 from the README, the
[design system plan](plans/2026-11-design-system.md),
`docs/contrast.md` and the existing tests. design_system has no deep review
yet, so these are a first guess at Jim's intent. Its production surface is
the preview site, <https://jimhillr-design-system.pages.dev>; the apps that
use the tokens check their own pages.

## DSGN-001: Every theme is readable for everyone

- Priority: must
- Why: Every app in the repo gets its colours from here, so one bad
  contrast pair would make text hard to read across all of them.
- Accept: all 20 combinations (five themes, light and dark, accessible
  mode off and on) meet WCAG 2.2 AA contrast, and AAA in accessible mode,
  for text, links, controls, focus rings and status colours; accessible
  mode also changes more than colour (larger text and targets); the
  check fails CI before a failing theme can merge.

## DSGN-002: An app theme can't make an app harder to use

- Priority: must
- Why: Apps like bear_down want their own brand colours, but a brand
  must never undo the accessibility the standard themes promise.
- Accept: an app's `design/theme.toml` extends a standard theme and
  overrides only primitives, semantic colours, fonts, radii and shadows;
  changing spacing, target sizes or focus rings is refused; its
  combinations pass the same contrast rules, so a brand colour that fails
  (such as the Bears orange on navy) fails the check.

## DSGN-003: People can pick a theme, and the choice sticks

- Priority: should
- Why: People read better in the scheme and contrast they choose, and
  shouldn't have to pick again on every visit.
- Accept: the theme picker offers the app's themes, light, dark or
  system colours, and accessible mode on, off or system; the choice is
  saved in `localStorage` and applied on the next visit; without a choice
  the page follows `prefers-color-scheme` and `prefers-contrast`; it works
  under a strict Content Security Policy with no inline script.

## DSGN-004: Apps, games and tools get the same values

- Priority: should
- Why: One source of truth means a game's colours match its web page and
  nothing drifts.
- Accept: the generated CSS, JSON (DTCG format) and Rust palette all hold
  the same value for every token and theme, and the generated files in the
  repo are up to date with the token sources.

## DSGN-005: The preview site shows every combination

- Priority: could
- Why: Jim and app authors compare themes side by side before picking or
  changing one.
- Accept: the preview site has one panel for each of the 20 combinations,
  showing text, links, buttons, inputs, focus rings, status colours,
  surfaces, the data palette and the type scale; it loads only its own
  files.
