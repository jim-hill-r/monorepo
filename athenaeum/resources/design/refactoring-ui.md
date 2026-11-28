# Refactoring UI

- **Owner:** Adam Wathan and Steve Schoger. *Refactoring UI*, self-published
  book, 2018.
- **Original:** <https://www.refactoringui.com/>, with a free chapter on
  [building your colour palette](https://www.refactoringui.com/previews/building-your-color-palette).
- **Licence:** copyright, summary only.
- **Added:** 2026-11-28, for the earthy theme redesign.

## What we take

- **You need more colours than you think,** as shades: 8 to 10 greys and 5 to
  10 shades of each brand and status colour, picked up front, not made up
  with `lighten()` as you go.
- **Greys should be tinted.** A warm theme's greys lean yellow-orange; pure
  neutral greys look cold next to them.
- **Avoid pure black and pure white.** Near-black text and an off-white page
  look softer and more finished.
- **Don't let the brand colour take over.** Most of the screen is greys; the
  primary colour marks actions and a few highlights. Big blocks of a
  saturated colour feel heavy.
- **Use fewer borders.** Separate with spacing, a background change or a
  shadow before reaching for a line, and keep lines light when you do.
- **Saturate as lightness moves away from the middle,** so very light and
  very dark shades don't look washed out.

## What we leave

The Tailwind-specific advice; our values come from the design system's
tokens, not utility classes.

## Where it shows up

The earthy theme's palette and cookbook's header, sidebar and card styles
(see the [README](README.md#how-we-apply-them-the-earthy-theme)).
