# Cahokia requirements

What Cahokia promises the people who use it to see their family's history.
The format and the rules are in
[standards/docs/evals.md](../../standards/docs/evals.md); the evals that
check each requirement are in [evals.toml](../evals.toml).

Drafted by an agent on 2026-11-26 from `core/README.md`, the
`Family tree visualization` epic in `core/issues/`, the
[OIDC error plan](plans/2026-11-prod-oidc-error.md) and the existing tests.
Cahokia has no deep review yet, so these are a first guess at Jim's intent.
Its colours are left out: the design system migration
(`design-system-phase-3-move-cahokia` in `standards/docs/issues/`) will replace
the current earthy palette.

## CAHO-001: A family tree is drawn so anyone can read it

- Priority: must
- Why: Seeing a family at a glance is the point of Cahokia.
- Accept: the family tree page draws every person as a box with their name
  and years; parents sit above their children and spouses side by side;
  lines join parents to children and spouses to each other; a long name is
  shortened with the full name on hover; a wide tree scrolls inside its own
  box instead of widening the page; until a user enters their own family,
  the made-up sample family is shown.

## CAHO-002: Odd or broken family data never breaks the tree

- Priority: should
- Why: Real family records have gaps, cycles and remarriages, and one bad
  record must not hide everyone else.
- Accept: a parent cycle, a person who is their own parent, a relationship
  to someone missing, and people with no relatives are all laid out without
  a crash; the layout is the same whatever order people were added in.

## CAHO-003: Anyone can sign in from any page

- Priority: must
- Why: A family's own tree will be private to the people signed in.
- Accept: every page shows a Login button and never "Authentication Error";
  signing in from any page sends Auth0 the site root as the redirect URL,
  which is one of the Cahokia application's Allowed Callback URLs.

## CAHO-004: The site loads only what it needs

- Priority: should
- Why: Family data is personal, so the page must not run scripts from
  anywhere else.
- Accept: the Content Security Policy blocks every source by default and
  allows only the site itself, Auth0, WebAssembly and the fonts it uses.

## CAHO-005: It is easy to move around the site

- Priority: should
- Why: The tree, the history of Cahokia and the about page should be a
  click apart.
- Accept: a menu button opens and closes the sidebar, which is hidden by
  default, stays open while moving between pages, and links to the family
  tree.

## CAHO-006: The same family tree on web, desktop and mobile

- Priority: should
- Why: Jim wants the tree on his phone and his laptop, not only in a
  browser.
- Accept: the desktop and mobile apps have the same family tree page as the
  web app, drawn from the same shared component.

## CAHO-007: A person can explore the tree around one relative

- Priority: could
- Why: In a large family, people want to follow one line.
- Accept: the tree can be panned and zoomed, and picking a person
  highlights their ancestors and descendants.
