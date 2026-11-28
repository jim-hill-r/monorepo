---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Feedback on every change
added: 2026-11-27
---

The shared `standards/tasks/docs.toml` that docs-only projects (athenaeum,
bogan, psy_ents, royalty) include makes `ci`, `test` and `evals` print a
message and pass. The [deploy feedback plan](../plans/2026-11-deploy-feedback.md)
says the "Content and docs" kind gets Markdown and link checks before merge,
so a broken relative link or bad Marp front matter in those projects merges
today without any signal.

Make the docs `ci` task run a Markdown link check over the project's `.md`
files (relative links and anchors only, so it needs no network), using a tool
that is already in the repo's toolchain or on the tech radar if one fits, and
fail on a broken link. Coordinate with the mise migration, which owns
`standards/tasks/`.
