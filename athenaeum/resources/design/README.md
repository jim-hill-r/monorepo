# Theme and colour design references

The sources we lean on when we design or change a design system theme
(`design_system/tokens/themes/`) or an app's look. Agents read the note for
a source before using its ideas, so every theme is built the same way.

The rules live in the design system, not here: the token model and the
contrast pairs in [design_system/](../../../design_system/README.md), and the
plan in
[design_system/docs/plans/2026-11-design-system.md](../../../design_system/docs/plans/2026-11-design-system.md).
CI proves contrast; these notes are about the part CI can't check, whether
a theme is pleasant to look at. When a note and the design system disagree,
the design system wins.

Each note uses the shape in
[code_quality/README.md](../code_quality/README.md#adding-a-source):
owner, original, licence, what we take, what we leave, where it shows up.

## Sources

| Note | Owner | Use it for |
| --- | --- | --- |
| [Refactoring UI](refactoring-ui.md) | Adam Wathan and Steve Schoger | Practical rules: shades not single colours, warm greys, accent used sparingly, no pure black or white |
| [OKLCH in CSS](oklch.md) | Evil Martians | Picking colours by perceived lightness, so a palette's steps look even |
| [Radix Colors](radix-colors.md) | WorkOS (Radix) | A 12-step scale where each step has a job: backgrounds, borders, solids, text |
| [Designing accessible color systems](stripe-accessible-color.md) | Stripe | Building a palette whose contrast is predictable while it still looks on-brand |
| [Generating colour palettes](strom-color-palettes.md) | Matthew Strom | Turning a few brand colours into full scales, in code |
| [Material 3 colour system](material-color.md) | Google | Tonal palettes and roles; low-chroma "neutral variant" surfaces |
| [Designing beautiful shadows](designing-shadows.md) | Josh W. Comeau | Soft, layered, tinted shadows instead of one dark blur |

## Tools

All free in the browser, nothing to install:

- [oklch.com](https://oklch.com/) (Evil Martians): an OKLCH picker that shows
  the sRGB gamut edge.
- [Huetone](https://huetone.ardov.me/): edit a whole palette in LCH and see
  its contrast (WCAG and APCA) at once.
- [Leonardo](https://leonardocolor.io/) (Adobe): generate a scale from target
  contrast ratios instead of from lightness.

## How we apply them: the earthy theme

The earthy theme is cookbook's and cahokia's brand. On 2026-11-28 Jim found
it "still pretty harsh", so it was redone with these notes. The checklist
we used, which any theme can follow:

1. **One hue family, low chroma.** Every earthy colour sits between OKLCH hue
   40 (terracotta) and 80 (sand), and surfaces stay under chroma 0.015. The
   old accent, a burnt orange at chroma 0.10, became a dusty terracotta at
   0.085 (Refactoring UI, OKLCH).
2. **No pure white or black.** Raised surfaces are warm paper (`#fffbf6`),
   text on the accent is cream (`#fdf8f0`), and text is a dark walnut, not
   black (Refactoring UI).
3. **Accent is a highlight, not a fill.** The app header, sidebar links and
   tags were solid accent blocks; now the accent marks the title, the one
   primary button per view and the hover state. Large areas use the
   background and surface steps (Radix's step jobs, Material's roles).
4. **Quiet borders.** Decorative lines use the low-contrast `border` token at
   1 px; `border-strong` (3:1) is kept for controls that need it (Radix).
5. **Soft, tinted, layered shadows.** Two small shadows tinted with the
   theme's brown at low opacity, instead of one dark blur (Comeau).
6. **Calm motion.** Links no longer jump sideways or grow on hover; a colour
   or border change is enough.
7. **Contrast is still proven.** `design_system_cli check` passes every pair
   in light, dark and accessible mode, and cookbook's axe tests run every
   theme (Stripe: pick for beauty, verify by number).
