# OKLCH in CSS

- **Owner:** Andrey Sitnik and Travis Turner, Evil Martians.
- **Original:** [OKLCH in CSS: why we moved from RGB and HSL](https://evilmartians.com/chronicles/oklch-in-css-why-quit-rgb-hsl)
  (2022), and the follow-up
  [Exploring the OKLCH ecosystem and its tools](https://evilmartians.com/chronicles/exploring-the-oklch-ecosystem-and-its-tools).
  Picker: [oklch.com](https://oklch.com/).
- **Licence:** copyright, summary only.
- **Added:** 2026-11-28, for the earthy theme redesign.

## What we take

- **Choose colours in OKLCH** (lightness, chroma, hue). Its lightness matches
  what eyes see, unlike HSL, where yellow and blue at the same "50%" look
  nothing alike. Two colours with the same L have about the same contrast
  against a background.
- **Chroma is the harshness dial.** A palette feels calm when its chroma
  stays low and steady; one high-chroma colour among low ones is what reads
  as "loud". Keep surfaces near 0, accents moderate.
- **Hold hue fixed and step lightness** to make a scale, so every shade of a
  colour looks like the same colour.
- Watch the sRGB gamut edge: high chroma at extreme lightness falls outside
  it and gets clipped.

## What we leave

Writing `oklch()` in our CSS. The design system emits hex so every app,
game and native palette reads the same values; we only design in OKLCH and
convert.

## Where it shows up

The earthy theme's primitives (`design_system/tokens/themes/earthy.toml`)
were picked as OKLCH values within hue 40 to 80.
