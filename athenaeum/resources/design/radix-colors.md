# Radix Colors

- **Owner:** WorkOS (the Radix project).
- **Original:** [Understanding the scale](https://www.radix-ui.com/colors/docs/palette-composition/understanding-the-scale)
  and [Composing a palette](https://www.radix-ui.com/colors/docs/palette-composition/composing-a-palette).
- **Licence:** the colour values and code are MIT; the docs are summarised here.
- **Added:** 2026-11-28, for the earthy theme redesign.

## What we take

- **Each step of a scale has a job.** Steps 1 and 2 are app and subtle
  backgrounds, 3 to 5 component backgrounds and their hover and pressed
  states, 6 to 8 borders (subtle to strong), 9 and 10 solid fills and their
  hover, 11 and 12 low- and high-contrast text.
- So a theme should never use a solid (step 9) where a background (1 to 3)
  belongs, which is exactly what made earthy feel heavy: the solid accent
  filled the header and every hovered link.
- **Pair a brand scale with a matching grey** (Radix's "sand" and "mauve"
  greys exist for warm and cool brands).

## What we leave

The 12-step scales themselves: our token set is semantic (`bg`, `surface`,
`border`, `accent`...), not numbered, so we map Radix's jobs onto our roles.

## Where it shows up

Our roles map to Radix steps roughly as `bg` 1, `surface` 2,
`surface-raised` 3, `border` 6, `border-strong` 8, `accent` 9,
`accent-hover` 10, `text-muted` 11, `text` 12.
