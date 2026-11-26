# Contributing to design_system/preview

The page is built by `src/main.rs` from `../tokens` (through
`design_system_model`), `../dist`, `../js/theme.js` and `../dioxus/fonts`.
Its own layout is `src/preview.css`, which uses only `--ds-` tokens so each
panel looks the way an app in that theme would.

Run `mise run ci` before committing, and `mise run dev` to look at the page.
See "Dev commands" in the root `AGENTS.md` for the current task runner.
