# Designing accessible color systems

- **Owner:** Daryl Koopersmith and Wilson Miner, Stripe.
- **Original:** [Designing accessible color systems](https://stripe.com/blog/accessible-color-systems)
  (Stripe blog, 2019).
- **Licence:** copyright, summary only.
- **Added:** 2026-11-28, for the earthy theme redesign.

## What we take

- **Make contrast predictable by level.** Build every hue's scale on the same
  perceptual lightness levels, so "any colour at level 5 or darker passes on
  white" holds across hues and designers don't have to check each pair.
- **Adjust hue as lightness changes** so dark shades don't go muddy and light
  ones don't go neon; the colour should still look like itself.
- **Accessible and beautiful are not a trade-off** when the system is tuned
  with a tool that shows both at once.

## What we leave

Stripe's in-house tooling; our CI check (`design_system_cli check`) and the
free tools in the [README](README.md#tools) do the same job.

## Where it shows up

The earthy status colours (success, warning, danger, info) sit at the same
OKLCH lightness (about 0.5 in light mode, 0.72 in dark), so they pass
contrast together and look equally weighted.
