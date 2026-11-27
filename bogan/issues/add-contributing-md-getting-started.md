---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

Add `bogan/CONTRIBUTING.md` with a `Getting Started` section, as
[standards/docs/documentation.md](../../standards/docs/documentation.md)
requires of every project. Bogan is docs-only today, so it should say so:
there is no toolchain beyond mise, `mise run ci` runs the shared
`standards/tasks/docs.toml` tasks (which check nothing yet), and where new
design docs go (`docs/`). The audit doesn't catch this yet (see the
standards issue to port DOC-001 to DOC-004 into `audit.sh`).
