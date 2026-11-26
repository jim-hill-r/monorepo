# Snapshot fixtures

A tiny fake monorepo for `pane_snapshot`'s tests. Files end in `.fixture` so
that agents and tools looking for real `ISSUES.md`, `mise.toml` or README
files in the repo don't pick these up, and every `TODO`/`FIX` here is tagged
`agent-ignore`. The tests copy them into a temporary directory under their
real names before walking it.
