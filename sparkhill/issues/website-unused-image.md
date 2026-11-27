---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

`website/static/images/better-world.webp` (74 KB, the largest file in
`static/`) is not used by any page, yet the export copies it and
`../cloudflare` uploads it. Delete it, and add a test that every file in
`static/images/` is referenced by a page, so unused images don't come back.
