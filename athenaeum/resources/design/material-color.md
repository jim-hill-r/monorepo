# Material 3 colour system

- **Owner:** Google.
- **Original:** [Material Design 3: colour system](https://m3.material.io/styles/color/system/overview),
  and the open source [material-color-utilities](https://github.com/material-foundation/material-color-utilities)
  (Apache 2.0).
- **Licence:** docs summarised here; the library is Apache 2.0.
- **Added:** 2026-11-28, for the earthy theme redesign.

## What we take

- **Colour roles over raw colours,** each with an "on" partner (`primary` and
  `on-primary`), the same idea as our `accent` and `on-accent`.
- **Tonal palettes:** each key colour becomes a range of tones at fixed
  perceptual lightness (HCT), and light and dark themes pick different
  tones from the same palette.
- **Neutral surfaces carry a trace of the brand hue** ("neutral variant"),
  which makes a whole screen feel on-brand without colouring it.
- Dark themes use lighter, less saturated tones of the accent, not the light
  theme's accent.

## What we leave

Dynamic colour from a wallpaper and the full role list (containers,
tertiary...); our token set is smaller on purpose.

## Where it shows up

Earthy dark mode's accent is a lighter, softer tone of the same terracotta
(`#dca37d`), and its surfaces are warm browns rather than greys.
