# design_system

One design system for every app in the monorepo: five themes (earthy, airy,
professional, colorful and gamey 8-bit), each in light and dark, each with an
accessible mode on top. That is 20 combinations, and CI proves every one
meets WCAG 2.2 contrast (AA, or AAA in accessible mode) before it merges.
Apps can also define their own theme that extends a standard one, for
example a Chicago Bears theme for bear_down.

The plan, with the reasons behind each choice, is
[standards/docs/plans/2026-11-design-system.md](../standards/docs/plans/2026-11-design-system.md).
The latest contrast results are in [docs/contrast.md](docs/contrast.md).

## Status

Phase 1 (the source and the check) is done: the token files, the model and
CLI, the generated CSS, JSON and Rust outputs, custom app themes, the
contrast check, the Dioxus picker with self-hosted fonts, and the preview
site at <https://design-system.pages.dev>. Next, apps adopt it, pane and
bear_down first.

## Using it

Apps read **semantic tokens** only: CSS custom properties named `--ds-...`.

```html
<html data-theme="professional">
  <link rel="stylesheet" href="design_system/dist/tokens.css">
  <link rel="stylesheet" href="design_system/dist/base.css">
```

```css
.card {
  background: var(--ds-color-surface-raised);
  border: var(--ds-border-width) solid var(--ds-color-border);
  border-radius: var(--ds-radius-md);
  padding: var(--ds-space-4);
  box-shadow: var(--ds-shadow-1);
}
```

Three attributes on `<html>` pick the combination:

| Attribute | Values | Without it |
| --- | --- | --- |
| `data-theme` | `earthy`, `airy`, `professional`, `colorful`, `gamey`, or an app theme | required |
| `data-scheme` | `light`, `dark` | follows `prefers-color-scheme` |
| `data-a11y` | `on`, `off` | on when `prefers-contrast: more` |

`prefers-reduced-motion` turns motion off in every mode, and Windows high
contrast (`forced-colors: active`) swaps the colour tokens for system colours.

**Dioxus 0.7 apps** use the `design_system_dioxus` crate, which embeds
the CSS, the fonts and the theme script, so nothing is copied:

```rust
use design_system_dioxus::{Stylesheets, ThemePicker};

rsx! {
    Stylesheets { default_theme: "professional" }
    ThemePicker {}
}
```

`ThemePicker` is a labelled form (theme; colours: system, light or dark;
accessible mode: system, on or off) that saves the choice in `localStorage`.
`Stylesheets` applies the saved choice once the app starts. To apply it
before first paint, also inline `design_system_dioxus::head_script(...)` in
the app's `index.html`. Pages without Dioxus inline `js/theme.js` the same
way, with `data-ds-default` (and optionally `data-ds-themes`) on `<html>`.

**Games and native UI** use the `design_system_palette` crate, which has every
value as a constant and no dependencies:

```rust
use design_system_palette::{GAMEY, Scheme};
let t = GAMEY.tokens(Scheme::Dark, false);
let bg = bevy::color::Color::srgba_u8(t.bg.0, t.bg.1, t.bg.2, t.bg.3);
```

**Other tools** can read `dist/tokens.json`, in the W3C Design Tokens (DTCG)
format.

### Tokens

| Group | Tokens |
| --- | --- |
| Surfaces | `--ds-color-bg`, `-surface`, `-surface-raised`, `-border` (decorative), `-border-strong` (controls) |
| Text | `--ds-color-text`, `-text-muted`, `-link` |
| Roles | `--ds-color-accent`, `-accent-hover`, `-on-accent`, `-focus`; `-success`, `-warning`, `-danger`, `-info`, each with `-on-<status>` |
| Data | `--ds-color-data-1` to `-data-8`, a categorical palette |
| Type | `--ds-font-body`, `-font-heading`, `-font-mono`; `--ds-text-xs` to `-text-3xl`; `--ds-leading-body`, `-leading-heading`; `--ds-weight-body`, `-weight-strong` |
| Space and shape | `--ds-space-0` to `-space-8` (4 px grid); `--ds-radius-sm`, `-md`, `-lg`, `-pill`; `--ds-shadow-1` to `-3`; `--ds-border-width` |
| Interaction | `--ds-focus-width`, `-focus-offset`, `--ds-target-min`, `--ds-motion-fast`, `-motion-slow`, `--ds-ease` |

## Custom app themes

An app writes `<app>/design/theme.toml`, naming the standard theme it
`extends` and overriding only what it needs:

```toml
name = "bears"
label = "Chicago Bears"
extends = "colorful"

[primitives]
navy = "#0b162a"
orange = "#c83803"

[light.color]
accent = "orange"
text = "navy"

[dark.color]
bg = "navy"
accent = "#ff8f5c"        # the brand orange is only 3.5:1 on navy

[accessible.light.color]
accent = "#a32e02"        # 7.1:1 on white, for AAA
```

It may override primitives, semantic colours, fonts, radii and shadows, but
not spacing, target sizes, focus rings or anything accessible mode turns off.
`schemes = ["light"]` makes a light-only theme, `offer_standard = false`
hides the standard themes from the app's picker, and `rust = true` also
writes `design/theme.rs` for a game. `mise run render` here writes the app's
`design/theme.css` (load it after `tokens.css`), and the check covers every
`*/design/theme.toml` in the repo with the same rules as the standard themes.

## Layout

```text
tokens/
  base.toml          spacing, type scale, radii, motion: shared by all themes
  modes.toml         what accessible mode changes
  pairs.toml         which foreground sits on which background, and the minimum ratio
  themes/*.toml      the five themes: primitives, then [light.color] and [dark.color]
css/elements.css     element defaults, after the reset, in dist/base.css
js/theme.js          applies the saved theme choice; inline it in <head>
model/               design_system_model: parse, resolve, contrast maths
cli/                 design_system_cli: render and check
palette/             design_system_palette: the values as Rust constants (generated)
dioxus/              design_system_dioxus: Stylesheets, ThemePicker, use_theme for Dioxus 0.7
dioxus/fonts/        Atkinson Hyperlegible and Press Start 2P (SIL OFL), latin subsets
preview/             the preview site (its own project, deployed to Cloudflare Pages)
dist/                tokens.css, base.css, tokens.json (generated, committed)
docs/contrast.md     every checked pair (generated, committed)
```

Everything under `dist/`, `palette/src/generated.rs` and `docs/contrast.md`
is generated: edit the TOML, then run `mise run render`. CI fails if they
don't match.

The fonts live in `dioxus/fonts/` rather than a top-level `fonts/` (as the
plan sketched) because Dioxus's `asset!` only reads files inside the crate
that uses it. Other stacks copy them from there, as the preview does.
