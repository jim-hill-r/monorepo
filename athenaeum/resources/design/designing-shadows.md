# Designing beautiful shadows in CSS

- **Owner:** Josh W. Comeau.
- **Original:** [Designing beautiful shadows in CSS](https://www.joshwcomeau.com/css/designing-shadows/)
  and his [shadow palette generator](https://www.joshwcomeau.com/shadow-palette/).
- **Licence:** copyright, summary only.
- **Added:** 2026-11-28, for the earthy theme redesign.

## What we take

- **Layer two or more small shadows** instead of one big blur; it looks like
  real light and stays soft.
- **Tint shadows with the background's hue** at low opacity; a grey or black
  shadow on a warm page looks dirty.
- **Elevation is consistent:** higher elements get longer offsets and more
  blur, from one light source.

## What we leave

Animating shadows on hover with `filter: drop-shadow`; our motion tokens are
off in accessible mode, and a colour change is enough feedback.

## Where it shows up

The earthy theme's `shadow-1` to `shadow-3` tokens: two layers each, tinted
`rgb(74 52 32)` at 6 to 10% opacity.
