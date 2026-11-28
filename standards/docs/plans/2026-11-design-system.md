# Plan: one design system for every app

Started 2026-11-26 from Jim's request:

> We also need to make a plan for a design system that all our apps can use.
> It should include a dark mode, a light mode, and an accessible mode. It
> should have five themes: earthy, airy, professional, colorful, and 8bit
> gamey.

And, later the same day:

> Our design system should also allow for us to override one of the
> standard themes with custom theme as defined in the app. The custom themes
> would typically to match another companies branding such as Chicago bears
> theme.

Status: **approved** by Jim (PR #657, 2026-11-26), with the recommended
answer to every question (see [Decisions](#decisions)). The work items are
issue files in the root [issues/](../issues/) with `epic: Design
system`, one group per phase.

## Goal

Every app in the monorepo looks like it belongs to one family, and each can
be shown in any of five themes and three modes without its own CSS changing.

- **One source of truth.** Colours, type, spacing, radii, shadows and motion
  are written once, as data, in a new `design_system` project.
- **Every stack reads it.** Dioxus web, desktop and mobile apps, the old
  Perseus and Sycamore sites, and the Bevy games all get the same values in
  the form they use: CSS custom properties for anything with a browser, Rust
  constants for the games.
- **Five themes:** earthy, airy, professional, colorful and gamey (8-bit).
- **Three modes:** light, dark and accessible.
- **Custom app themes.** An app can override a standard theme with its own,
  for example to match another organisation's branding (a Chicago Bears
  theme for bear_down), and the same checks apply to it.
- **Accessibility is checked, not hoped for.** CI proves every theme and mode
  meets WCAG 2.2 contrast before it merges, and the apps' browser tests
  check the pages themselves.
- **$0.** Rust, files in the repo and the CI we already run. No new vendor.

## Where we start

A survey of every project with a user interface, on 2026-11-26:

| Project | Stack | Styling today | Light and dark |
| --- | --- | --- | --- |
| `cahokia/web`, `cahokia/ui`, `cahokia/desktop`, `cahokia/mobile` | Dioxus 0.7 (pinned) | Hand-written CSS loaded with `asset!`; hard-coded warm browns (`#3d2f1f`, `#d4a574`); the `sidebar`, `navbar` and `hero` sheets are copied between `ui` and `web` | Dark only |
| `cookbook/web` | Dioxus 0.7 | Six CSS files, 19 hard-coded colours, saddle-brown header (`#8b4513`) | Light only |
| `pane/web` | Dioxus 0.7 | One CSS file with custom properties (`--bg`, `--fg`, `--muted`, `--accent` ...) | Both, through `prefers-color-scheme` |
| `bear_down/web` | Dioxus 0.7 | One CSS file with custom properties in the team's navy and orange | Both, through `prefers-color-scheme` |
| `blueeel`, `sparkhill/webapp` | Dioxus 0.6 | Little or none | No |
| `sparkhill/website`, `profiles/jimhillr` | Perseus 0.4.2 and Sycamore 0.8 (Hold on the radar) | A compiled Tailwind file (1,105 lines) and a small theme file; both copy `standards/docs/reset.css` | No |
| `controlled_chaos` | Bevy 0.14 | `Color::srgb(...)` literals in `card.rs`, `game.rs`, `splash.rs` | Dark only |
| `starcraft` (on hold) | Bevy 0.15 | Sprite sheets | n/a |

What this tells us:

1. **CSS custom properties are the common ground.** Every browser-based app
   is either a Dioxus app that loads plain CSS files or a Perseus site that
   serves static CSS. None uses a CSS-in-Rust library, Tailwind at runtime or
   a Node build step for styles. A token file of custom properties works in
   all of them with no new dependency.
2. **Two apps already do it.** `pane/web` and `bear_down/web` define
   semantic custom properties and switch them for dark mode. Their names map
   one to one onto the semantic tokens below, so they are the first to move.
3. **Dioxus versions differ.** Most apps are on 0.7, cahokia's version is
   pinned, and two apps are on 0.6. So the shared part must be CSS, not a
   Dioxus component crate that would force one version on everyone.
4. **Bevy versions differ too** (0.14 and 0.15), so the games get colours as
   plain numbers, not as a `bevy::Color`, and convert them in one line.
5. **Accessibility testing exists in one place.** `cookbook/web` already
   runs `@axe-core/playwright`. Nothing checks contrast across themes.

## Shape of the system

### Three layers of tokens

| Layer | What it holds | Example | Who uses it |
| --- | --- | --- | --- |
| **Primitives** | Each theme's raw palette ramps and font stacks | `clay-700 = #8a4b1f` | Only the theme files |
| **Semantic** | What a value is for, the same names in every theme and mode | `--ds-color-text`, `--ds-color-surface`, `--ds-space-3` | Apps |
| **Component** | One app's own names, defined from semantic tokens | `--header-bg: var(--ds-color-surface-raised)` | That app |

Apps only ever read semantic tokens. That is what lets one app switch theme
or mode without touching its own CSS. The shared system ships no components
at first, only tokens and a base sheet (reset plus element defaults).

### Semantic tokens

All names start with `--ds-` so they never clash with an app's own
properties. The first set, small on purpose:

- **Colour, surfaces:** `bg`, `surface`, `surface-raised`, `border`
  (decorative), `border-strong` (controls, 3:1 against what it sits on).
- **Colour, text:** `text`, `text-muted`, `link`, `on-accent`.
- **Colour, roles:** `accent`, `accent-hover`, `focus`, `success`,
  `warning`, `danger`, `info`, each with an `-on` partner for text on it.
- **Colour, data:** `data-1` to `data-8`, a categorical palette for charts,
  map layers and game categories (controlled_chaos has eight card
  categories), distinct from each other and at least 3:1 against `surface`.
- **Type:** `font-body`, `font-heading`, `font-mono`, a size scale
  `text-xs` to `text-3xl` built from one base and ratio, `leading-body`,
  `leading-heading`, `weight-body`, `weight-strong`.
- **Space and shape:** `space-0` to `space-8` on a 4 px grid, `radius-sm`,
  `radius-md`, `radius-lg`, `radius-pill`, `shadow-1` to `shadow-3`,
  `border-width`.
- **Interaction:** `focus-width`, `focus-offset`, `target-min` (minimum hit
  size), `motion-fast`, `motion-slow`, `ease`.

Pane's `--bg`, `--fg`, `--muted`, `--line`, `--nav-bg`, `--accent` and
`--error` map to `bg`, `text`, `text-muted`, `border`, `surface`, `accent`
and `danger`. Bear_down's `--card` maps to `surface`.

### Themes

A theme is a personality: palette, fonts, corner shape, shadow and motion.
Each theme defines every semantic token for both light and dark.

| Theme | Feel | Palette | Type | Shape | Default for |
| --- | --- | --- | --- | --- | --- |
| **Earthy** | Warm, natural, handmade | Clay, bark, sand and olive | Serif headings (system serif stack), humanist sans body | Soft corners, warm low shadows | `cahokia`, `cookbook` (both already brown) |
| **Airy** | Light, calm, spacious | Sky blue, mist and white | System sans at lighter weights, looser line height | Large radii, almost no shadow, more spacing | `sparkhill`, `profiles/jimhillr` |
| **Professional** | Neutral, dense, quiet | Greys with one blue accent | System sans, tight scale | Small radii, thin borders | `pane`, `evals` and other tools |
| **Colorful** | Playful, bright | Magenta, violet, teal and sun yellow | Rounded system stack (`ui-rounded`) | Pill buttons, bold borders | `bear_down`, under its own Bears theme (below) |
| **Gamey (8-bit)** | Retro console, chunky | A small fixed palette of a few flat colours, no gradients | Pixel font for headings, monospace body | Square corners, 2 px hard borders, stepped "pixel" shadows, `image-rendering: pixelated` | `controlled_chaos`, `starcraft` |

Starting values for the colours that matter most, already checked against
the WCAG thresholds the CI check will use (normal text 4.5:1, controls
3:1). Ratios are against `bg`; every pair also passes on `surface`.

| Theme / mode | `bg` | `surface` | `text` | `text-muted` | `accent` | `on-accent` | `border-strong` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Earthy light | `#f7f1e8` | `#fffaf3` | `#2b2118` (14.0) | `#6b5a48` (5.9) | `#8a4b1f` (6.0) | `#ffffff` | `#8c7a66` (3.7) |
| Earthy dark | `#1f1812` | `#2a2119` | `#f3e7d8` (14.4) | `#bba891` (7.6) | `#e0a068` (7.8) | `#1f1812` | `#7d6a55` (3.4) |
| Airy light | `#ffffff` | `#f4f8fb` | `#1e2a33` (14.6) | `#566774` (5.9) | `#1f6fb2` (5.3) | `#ffffff` | `#7a8d9c` (3.4) |
| Airy dark | `#0f1720` | `#16212c` | `#e8f1f7` (15.8) | `#9fb2c1` (8.3) | `#7cc4f5` (9.5) | `#0f1720` | `#5a7184` (3.5) |
| Professional light | `#ffffff` | `#f6f7f9` | `#1d1f23` (16.5) | `#5c6370` (6.0) | `#2557d6` (6.2) | `#ffffff` | `#80868f` (3.6) |
| Professional dark | `#15171a` | `#1c1f23` | `#e6e8eb` (14.6) | `#9aa1ad` (6.9) | `#7ea6ff` (7.5) | `#15171a` | `#6a7280` (3.7) |
| Colorful light | `#fffdf7` | `#ffffff` | `#231942` (16.0) | `#5e548e` (6.6) | `#b5146f` (6.3) | `#ffffff` | `#9a7fb0` (3.4) |
| Colorful dark | `#1a1033` | `#251845` | `#fbf5ff` (16.8) | `#c9b8e8` (9.9) | `#ff6fb5` (7.0) | `#1a1033` | `#7a62a8` (3.6) |
| Gamey light | `#e0f8d0` | `#c8ecb4` | `#081820` (15.9) | `#2a5546` (7.4) | `#1d4a8c` (7.7) | `#ffffff` | `#081820` (15.9) |
| Gamey dark | `#000000` | `#1c1c3c` | `#fcfcfc` (20.5) | `#bcbcbc` (11.1) | `#f8b800` (11.8) | `#000000` | `#fcfcfc` (20.5) |

Professional is pane's current palette with stronger borders, so pane
changes the least. The remaining tokens (status colours, the data palette,
hover states) are filled in during phase 1 under the same check.

### Modes, and how they combine with themes

**Light** and **dark** are colour schemes: every theme has both.

**Accessible** is not a third colour scheme but a modifier that applies on
top of light or dark in any theme. Treating it as a modifier keeps the
choice people actually need (some people with low vision want a dark
accessible view, others a light one) and keeps a theme's personality where
it does no harm. Accessible mode:

- raises contrast to WCAG **AAA**: 7:1 for all text including `text-muted`
  and `link`, 4.5:1 for `border-strong`, focus rings and icons;
- makes focus rings wider (`focus-width` 3 px with an offset) and always
  visible, not only on keyboard focus;
- underlines every link and never signals state by colour alone;
- raises `target-min` to 44 px and the base font size by one step;
- turns off motion (`motion-*` become `0ms`), shadows and translucency;
- swaps decorative fonts for a highly legible one: the 8-bit pixel font and
  the earthy serif headings become Atkinson Hyperlegible (SIL Open Font
  License, self-hosted), and body text stays a plain sans;
- swaps the data palette for one that is safe for the common forms of
  colour blindness, with patterns or labels allowed alongside.

So there are **5 themes × 2 schemes × 2 (standard or accessible) = 20
combinations**, and CI checks all 20.

How a page picks its combination, in order:

1. **Theme:** the user's saved choice, else the app's default theme.
2. **Scheme:** the user's saved choice, else `prefers-color-scheme`, else
   light.
3. **Accessible:** the user's saved choice, else on when the browser reports
   `prefers-contrast: more`. `prefers-reduced-motion: reduce` turns off
   motion in every mode, not only accessible. Windows high contrast
   (`forced-colors: active`) is respected: the system's colours win and the
   tokens step aside.

The page records the result as three attributes on `<html>`:
`data-theme="earthy"`, `data-scheme="dark"` and `data-a11y="on"`. The token
sheet has one block per combination, keyed on those attributes, plus media
query defaults so a page with no script still follows the system scheme and
contrast setting. The choice is saved in `localStorage` (files before
databases: nothing on a server) and applied by a few lines of inline script
in the page head, before first paint, so there is no flash of the wrong
theme.

Every app gets a small settings control (theme, light, dark or system,
accessible on or off). In Dioxus 0.7 apps that is one shared component (see
below); the others write the three attributes themselves.

### Custom app themes

An app can define its own theme, usually to match another organisation's
branding (for example a Chicago Bears theme for bear_down), on top of one of
the five standard themes. Jim asked for this on 2026-11-26.

**How it works.** A custom theme lives in the app, not in `design_system`,
as `<app>/design/theme.toml`. It names the standard theme it `extends` and
overrides only what it needs. Everything it doesn't set (fonts, spacing,
shape, motion, status colours, the data palette, accessible-mode rules) comes
from the base theme, so a brand theme can be a dozen lines:

```toml
# bear_down/design/theme.toml
name = "bears"
label = "Chicago Bears"
extends = "colorful"         # fonts, shape and everything unset come from here

[primitives]
navy = "#0b162a"
orange = "#c83803"
orange-light = "#ff8f5c"      # the brand orange is only 3.5:1 on navy

[light.color]
accent = "orange"             # 5.2:1 on white
on-accent = "#ffffff"
text = "navy"

[dark.color]
bg = "navy"
surface = "#14223b"
accent = "orange-light"       # 8.0:1 on navy, so AAA too
on-accent = "navy"

[accessible.light.color]
accent = "#a32e02"            # 7.1:1 on white, for AAA
```

**What it may override:** any primitive and any semantic colour, the fonts
(self-hosted in the app, under an open licence, or a system stack), and the
radii and shadows. It may not change the spacing scale, `target-min`,
`focus-width` or anything accessible mode turns off, so a brand can't make
an app less usable.

**What it must pass:** exactly what a standard theme passes. The contrast
check resolves the custom theme over its base and checks its four
combinations (light, dark, each standard and accessible). A brand colour that
fails, as the Bears orange does as text on navy, fails CI until the theme
gives a lighter or darker shade for that scheme. If a brand has no dark
look, the theme can set `schemes = ["light"]` and the app shows light only;
accessible mode is always available and never optional.

**What it generates:** `design_system_cli render --app <app>` writes
`<app>/design/theme.css`, one block per combination under
`data-theme="bears"`, committed and checked like `dist/tokens.css`, and for a
game a `theme.rs` of constants. The app loads it after `tokens.css`.

**In the picker:** the custom theme is the app's default and is listed
first. The app chooses, in the same file, whether the standard themes are
also offered (`offer_standard = true`, the default), so people can still
switch to plainer themes.

**Branding care:** colours and fonts are fine to match. Logos, team marks
and names are trademarks: an app uses them only where it already has the
right to, and the design system never ships them.

## The source and what it generates

### Files

A new project, `design_system/`, with the usual `mise.toml`, `royalty.toml`,
`issues/` folder, README and CONTRIBUTING:

```text
design_system/
  tokens/
    base.toml          # spacing, type scale, radii, motion: shared by all themes
    modes.toml         # what accessible mode changes, as rules and overrides
    themes/
      earthy.toml      # primitives + semantic tokens, [light] and [dark]
      airy.toml
      professional.toml
      colorful.toml
      gamey.toml
    pairs.toml         # which foreground sits on which background, and the minimum ratio
  fonts/               # self-hosted OFL fonts: a pixel font, Atkinson Hyperlegible
  model/               # Rust crate design_system_model: parse, resolve, check contrast
  cli/                 # Rust crate design_system_cli: render and check
  palette/             # Rust crate design_system_palette: the values as Rust constants
  dioxus/              # Rust crate design_system_dioxus: theme picker for Dioxus 0.7
  dist/
    tokens.css         # generated, committed, checked to match the TOML
    base.css           # reset (replaces the copies of standards/docs/reset.css) and element defaults
```

TOML is the source because the repo already keeps its data as TOML or JSON
(files before databases), it allows comments explaining why a colour was
chosen, and Rust reads it with `serde`. A theme file looks like this:

```toml
# design_system/tokens/themes/earthy.toml
name = "earthy"
description = "Warm, natural, handmade"

[fonts]
heading = "ui-serif, 'Iowan Old Style', Georgia, serif"
body = "ui-sans-serif, system-ui, 'Segoe UI', Roboto, sans-serif"

[primitives]
clay-700 = "#8a4b1f"
clay-300 = "#e0a068"
bark-900 = "#2b2118"
sand-50 = "#f7f1e8"

[light.color]
bg = "sand-50"
text = "bark-900"
accent = "clay-700"
on-accent = "#ffffff"

[dark.color]
bg = "#1f1812"
text = "#f3e7d8"
accent = "clay-300"
on-accent = "#1f1812"
```

### Generated outputs

`design_system_cli render` writes, from the TOML:

- **`dist/tokens.css`** for every browser stack: one rule per combination
  keyed on the `<html>` attributes, plus the media query defaults. It is
  committed, like the tech radar tables, and `design_system` CI fails when it
  doesn't match the TOML. Committing it is what lets Dioxus `asset!`,
  Perseus static folders and plain HTML all point at a real file with no
  build step of their own.
- **`design_system_palette`** (in `design_system/palette/`), a tiny Rust crate with no dependencies, whose
  generated `src/generated.rs` holds every resolved value as constants,
  for example `earthy::dark::ACCENT: Rgba = Rgba(0xe0, 0xa0, 0x68, 0xff)`.
  Bevy games turn one into a colour with `Color::srgba_u8(r, g, b, a)`,
  which both Bevy 0.14 and 0.15 have, so no Bevy version is forced on
  anyone. It also has the type scale and spacing as numbers for game UI.
- **`dist/tokens.json`** in the W3C Design Tokens (DTCG) format, so any
  outside tool that reads design tokens (a Figma plugin, a docs site) can
  use them without knowing our TOML. Not needed by anything yet; it costs
  one function.

### How each stack uses it

| Stack | How it gets the tokens | Theme switching |
| --- | --- | --- |
| Dioxus 0.7 web (cookbook, pane, bear_down, cahokia) | `design_system_dioxus::Stylesheets {}`, which embeds `dist/tokens.css` and `base.css` with `include_str!` and puts them in the page head, so no app copies a file | `design_system_dioxus::ThemePicker` and `use_theme()`; reads and writes the `<html>` attributes and `localStorage` |
| Dioxus 0.7 desktop and mobile (cahokia) | The same assets; the webview supports custom properties | Same component; the saved choice uses the platform's storage |
| Dioxus 0.6 (blueeel, sparkhill/webapp) | Link the CSS files | Write the three attributes directly; the component comes when they move to 0.7 |
| Perseus and Sycamore 0.8 (sparkhill/website, profiles) | Copy of `dist/` into `static/css/` by their `build` task | Inline head script only; no new Sycamore code (Hold on the radar) |
| Bevy 0.14 and 0.15 (controlled_chaos, starcraft) | Path dependency on `design_system_palette` | A `Resource` holding the current theme and scheme; gamey is the default, accessible mode available from the settings screen |
| Playwright specs | Read `dist/tokens.json` to know which combinations exist | Set the attributes before each page load |

Using `design_system` as a relative path dependency follows the existing rule
for shared code (`auth_sdk`, [mise.md](../mise.md)). Each consuming project
adds `# affected-by: design_system` to its `mise.toml`, so `affected.sh`
reruns its CI whenever the tokens change.

## Checks in CI

### Contrast check (the gate)

`design_system_cli check` runs in `design_system`'s `ci` task, so it runs in
the git hooks and PR CI whenever a token file changes. For each of the 20
combinations it resolves every token and checks every pair listed in
`pairs.toml`:

| Pair | Standard | Accessible |
| --- | --- | --- |
| `text`, `text-muted`, `link` on `bg`, `surface`, `surface-raised` | 4.5:1 (AA) | 7:1 (AAA) |
| `on-accent` on `accent` and `accent-hover`; each status `-on` on its status colour | 4.5:1 | 7:1 |
| `border-strong`, `focus` on `bg` and `surface` (WCAG 1.4.11 non-text) | 3:1 | 4.5:1 |
| `data-1` to `data-8` on `surface` | 3:1 | 3:1, plus a colour-blind distance check |
| Large headings (24 px and up) | 3:1 | 4.5:1 |

It also checks that every theme defines every semantic token in both
schemes, that references resolve, and that `dist/` matches. Ratios use the
WCAG 2.2 relative luminance formula, in our own Rust, about forty lines and
unit-tested against the published examples. It prints the APCA lightness
contrast next to each ratio as information only, since WCAG 3 is not final.
Results are written to `design_system/docs/contrast.md` (generated,
committed), a table Jim can read without running anything.

Custom app themes are checked the same way: `design_system` CI checks
every `*/design/theme.toml` in the repo, and each app's own CI runs the check
on its theme, so a change on either side is caught. Each app with a custom
theme lists it in `affected-by`, so the tokens changing reruns its check.

### Using the tokens (lint)

A new `//:audit` rule warns when a CSS file in a project that has adopted the
design system contains a raw colour (`#...`, `rgb(...)`, `hsl(...)`) outside
`design_system/`, and when a Rust file in an adopted game builds a colour
from literals. It is a warning until every app in the adoption order has
moved, then an error, the same path the cost check takes.

Done 2026-11-27, straight as an error: by then phases 2 to 4 had moved
every adopted project to tokens and none had a raw colour left, so a
warning stage would have guarded nothing. The rule is
`standards/scripts/design-check.sh`, run by `//:audit`: DS-001 for CSS
(hex and the colour functions, `color-mix()` included) and DS-002 for Rust
colour constructors with literal arguments. A project counts as adopted
when it has a Cargo dependency on a `design_system` crate, a
`design/theme.toml`, or an `# affected-by:` line naming `design_system` in
its `mise.toml` (how the CSS-only apps of phase 5 opt in), so each phase 5
app is checked from the PR that adopts it. A line that needs an exact colour
ends with a `ds-allow-raw-colour` comment and a reason.

### Checking the real pages

Contrast between tokens doesn't prove a page is readable: text can sit on an
image, or an app can forget a token. So each Dioxus web app's Playwright
smoke spec gains one test that loads its main pages in all 20 combinations
and runs `@axe-core/playwright` with the `wcag2aa` tags (and `wcag2aaa`
contrast in accessible mode). `cookbook/web` already depends on axe, so it is
the model. These run where Playwright already runs: PR CI and the preview
smoke UAT from decision 0001.

## Adoption order

Each step is one PR, small enough to review, and none blocks the others once
phase 1 is done.

### Phase 1: the source and the check

- The `design_system` project: the five theme files, `base.toml`,
  `modes.toml`, `pairs.toml`, and the `model` and `cli` crates with `render`
  and `check`, unit tests for the contrast maths, and generated `dist/` and
  `docs/contrast.md`.
- Fill in every semantic token for all 20 combinations until `check` passes.
- A static preview page, `design_system/preview/index.html`, showing every
  token in every combination, built by the `release` task and deployed to
  Cloudflare Pages as `design-system` through `free-tiers.sh`. It is the
  place Jim looks at the themes.

### Phase 2: the two apps that already use custom properties

1. **pane/web** (professional): map its properties onto `--ds-` tokens, drop
   its own colour values, add the theme picker. First user of
   `design_system_dioxus`.
2. **bear_down/web**: its Bears theme, extending colorful, becomes the
   first custom app theme, so it proves the override path early.

### Phase 3: the apps with hard-coded colours

3. **cookbook/web** (earthy): replace its 19 hard-coded colours, and add the
   20-combination axe test (it already has axe).
4. **cahokia** (earthy): `web`, `ui`, `desktop` and `mobile` together, which
   also removes the duplicated `sidebar`, `navbar` and `hero` sheets.

### Phase 4: the games

5. **controlled_chaos** (gamey): replace its `Color::srgb(...)` literals with
   `design_system_palette`, card categories from `data-1` to `data-8`, and add
   accessible mode to its settings.
   Done 2026-11-27: the light or dark choice and accessible mode are on its
   start screen, which is its only settings screen.
6. **starcraft** when it comes off hold.

### Phase 5: the rest

7. **blueeel** and **sparkhill/webapp** (Dioxus 0.6): CSS only.
8. **sparkhill/website** and **profiles/jimhillr** (Perseus, airy): link the
   CSS, replacing their copies of the reset. The compiled Tailwind file in
   sparkhill stays until that site is rebuilt in Dioxus.
9. Turn the raw-colour lint from a warning into an error. Done
   2026-11-27: it shipped as an error (see [Using the tokens](#using-the-tokens-lint)).

## Tech radar and Claude support

Choices, weighed by the radar's selection criteria (cost first, then Claude
support, then fewest vendors):

| Option | Cost | Claude support | Verdict |
| --- | --- | --- | --- |
| **CSS custom properties generated from TOML by our own Rust** | $0, no new tool | Claude knows CSS custom properties, WCAG maths, TOML and serde very well; the rust-analyzer plugin covers the code | **Chosen.** Fits every stack, adds nothing to install. |
| Style Dictionary (Amazon, Node) | $0 | Well known to Claude | Not chosen: adds a Node build step to every app, and Node is allowed only for e2e tests ([typescript.md](../typescript.md)). Our DTCG JSON export keeps the door open. |
| Tailwind CSS | $0 | Very well known to Claude | Not chosen: needs Node or its standalone binary in every app's build, and utility classes would tie markup to one theme engine. Its theming also ends up as custom properties. |
| Open Props, Pico CSS or another ready-made CSS kit | $0 | Known to Claude | Not chosen: they give one look, not five themes and an accessible mode, and we would still write the themes. Worth borrowing ideas from. |
| A Rust CSS-in-code crate (for example `stylist`) | $0 | Little | Not chosen: tied to particular frameworks and versions, and doesn't help Perseus or Bevy. |
| Figma or another hosted design tool | Free tier exists | MCP servers exist | Not needed: the preview page is the place to look at themes, and a new vendor needs Jim's sign-off. |

Fonts are self-hosted from `design_system/fonts/`, not loaded from a font
CDN, so no new vendor is involved and pages work offline. Both fonts are
under the SIL Open Font License.

The tech radar has a Trial blip, "Shared design tokens (design_system)",
under Techniques, sourced from this plan (Assess while it was proposed). It
moves to Adopt, with `pane/web` as its exemplar, when phase 2 lands.

## Cost of this plan

Nothing new. Rust on standard GitHub-hosted runners, files in the repo, and
one more static Cloudflare Pages project (the preview page) on the free plan,
created through `free-tiers.sh`. The fonts add about 100 KB to each app that
uses them, well within the Pages free limits. The contrast check runs in
well under a second; the 20-combination axe tests add about a minute to each
web app's Playwright run.

## Decisions

Jim, 2026-11-26: "Yes to all" to these recommended answers.

1. **Accessible mode is a modifier**, available in light and dark for every
   theme (20 combinations), not a single separate high-contrast theme.
2. **Accessible means WCAG AAA contrast** (7:1 text), wider focus rings,
   underlined links, no motion, 44 px targets and legible fonts; standard
   modes must meet AA.
3. **Source format is TOML** in `design_system/tokens/`, with a DTCG JSON
   export generated alongside.
4. **Generated `dist/tokens.css` is committed** and checked to match, like
   the tech radar tables, rather than built by each app.
5. **Every app ships all five themes and a picker**, each with a default:
   earthy for cahokia and cookbook, professional for pane and tools, colorful
   for bear_down until its own Bears theme lands, airy for sparkhill and profiles, gamey for the games.
6. **Custom app themes** live in `<app>/design/theme.toml`, extend one
   standard theme, may override colours, fonts, radii and shadows but not
   spacing, targets, focus or accessible-mode rules, and pass the same
   contrast check. They are the app's default, with the standard themes
   still offered unless the app turns them off.
7. **The user's choice is saved in the browser** (`localStorage`), never on
   a server, and follows the system's scheme and contrast settings until the
   user picks.
8. **Self-host two OFL fonts**: a pixel font for gamey headings and Atkinson
   Hyperlegible for accessible mode. Everything else uses system font
   stacks.
9. **Adoption order** as above: pane, bear_down, cookbook, cahokia,
   controlled_chaos, then the rest.
10. **The raw-colour lint warns first** and becomes an error after phase 4.
11. **A preview site** for the design system on Cloudflare Pages, free plan.

## Not doing (yet)

- **A shared component library** (buttons, cards, dialogs). Tokens and a base
  sheet come first; components follow once two apps have built the same one.
- **A theme editor** or user-defined themes.
- **Right-to-left layouts and print styles.**
- **Figma or other design-tool sync.** The DTCG export makes it possible
  later.
- **Rebuilding the Perseus sites.** They only link the CSS.
