---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

`webapp` has no Playwright tests (no `package.json`), so nothing checks the
pages in a browser: that "Go" at the end of the letter level opens `/word`,
that it opens `/congratulations` after words, or that tracing and "Try
again!" work. Add a Playwright suite like the other Dioxus apps
(standards/docs/testing.md) that drives the level with a short test sequence.
