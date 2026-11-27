---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

`psy_ents` has no `CONTRIBUTING.md`. `standards/docs/documentation.md` says
every project has one with a `Getting Started` section on installing the
toolchain, building and testing.

Add one that says the stories are Marp decks (`marp: true` front matter, the
`uncover` theme), how to preview one (the Marp VS Code extension, or
`npx @marp-team/marp-cli "The Psy Ents.md" --preview`), and that `mise run ci`
is the docs-only task from `standards/tasks/docs.toml`, which checks nothing
yet (see [docs-only-ci-checks-nothing](../../issues/docs-only-ci-checks-nothing.md)).
