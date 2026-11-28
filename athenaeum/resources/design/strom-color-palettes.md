# How to generate colour palettes for design systems

- **Owner:** Matthew Ström.
- **Original:** [How to generate color palettes for design systems](https://matthewstrom.com/writing/generating-color-palettes/).
- **Licence:** copyright, summary only.
- **Added:** 2026-11-28, for the earthy theme redesign.

## What we take

- **Generate scales in code from a few inputs** (a base hue and chroma, and
  target contrast levels) rather than hand-picking dozens of hexes; the
  output is consistent and easy to retune.
- **Aim at contrast targets, not lightness numbers,** so the scale lands on
  WCAG thresholds by construction.
- Keep chroma highest in the middle of a scale and lower at the ends.

## What we leave

The specific formulas; our themes are small enough to tune by hand with the
CI check as the safety net. Revisit if we add numbered scales.

## Where it shows up

The scratch script used to tune the earthy theme followed this approach
(target contrast first, then pick L and C in OKLCH).
