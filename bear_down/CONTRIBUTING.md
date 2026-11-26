# Contributing to bear_down

## Getting Started

The project has no code yet. The first code change adds a `mise.toml` that
includes the shared task file from `standards/tasks/`, following the repo-wide
move from Cast to mise. After that:

- Install the toolchain: `mise install` (in this directory)
- Build, lint and test everything: `mise run ci`
- Run only the tests: `mise run test`
- Serve the site locally: `mise run dev`

See "Dev commands" in the root `AGENTS.md` for the current task runner.

## Recording decisions

Decisions that shape the project go in `docs/decisions/` as numbered files
(`NNNN-short-title.md`) with Context, Decision and Consequences sections. To
change a decision, add a new record that supersedes the old one and mark the
old one `Superseded by NNNN` instead of rewriting it.
