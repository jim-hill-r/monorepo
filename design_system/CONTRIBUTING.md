# Contributing to design_system

## Getting started

`mise.toml` here covers the whole Cargo workspace (`model`, `cli`,
`palette`) with the shared Rust tasks, plus two of its own:

- Format, lint, build, test and run the contrast check: `mise run ci`
- Regenerate every output after editing the TOML: `mise run render`
- Only the contrast check, and that outputs are up to date: `mise run check`

See "Dev commands" in the root `AGENTS.md` for the current task runner.

## Changing a colour

1. Edit the theme file in `tokens/themes/`. Values are `#hex`, a name from
   the theme's `[primitives]`, or another semantic token (`link = "accent"`).
2. Run `mise run render`, then read the theme's table in `docs/contrast.md`.
3. If a pair fails, pick a lighter or darker shade for that scheme. Set it
   under `[accessible.<scheme>.color]` when only accessible mode needs it.

Every pair that is checked, and its minimum, is in `tokens/pairs.toml`. Add
a pair there when a new foreground and background are used together.

## Adding a token

Add its name to `COLOR_TOKENS` or `OTHER_TOKENS` in `model/src/spec.rs`, give
it a value in every theme (colours) or in `tokens/base.toml` (anything
else), run `mise run render`, and mention it in the README's token table.
Renaming or removing a token breaks apps: search the repo for `--ds-<name>`
first.

## Rules

- Apps read only semantic `--ds-` tokens, never a theme's primitives.
- Accessible mode is never optional and a custom theme can't weaken it.
- No new vendor: fonts are self-hosted, everything else is our own Rust.
