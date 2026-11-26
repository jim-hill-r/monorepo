# Contributing to bear_down

## Getting Started

bear_down uses mise, not Cast. `mise.toml` here covers the whole Cargo
workspace with the shared Rust tasks, and `web/mise.toml` adds the Dioxus
tooling for the web app. Both include a task file from `standards/tasks/`.

- Install the toolchain: `mise install` (here, and in `web/` for `dx`)
- Format check, lint, build and test everything: `mise run ci`
- Run only the tests: `mise run test`
- Serve the site locally: `mise run dev` (runs `dx serve` in `web/`)
- Release bundle of the site: `mise run release` in `web/`

When you change the `dioxus` version in `Cargo.lock`, change
`cargo:dioxus-cli` in `web/mise.toml` to match.

See "Dev commands" in the root `AGENTS.md` for the current task runner.

## Recording decisions

Decisions that shape the project go in `docs/decisions/` as numbered files
(`NNNN-short-title.md`) with Context, Decision and Consequences sections. To
change a decision, add a new record that supersedes the old one and mark the
old one `Superseded by NNNN` instead of rewriting it.
