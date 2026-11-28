---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Feedback on every change
order: 70
added: 2026-11-26
---

Add a Pages deploy and `@smoke` Playwright tests for `blueeel` (the older
top-level stub, not the Blue Eel app), matching cookbook, pane and cahokia.
Blue Eel itself (sparkhill/webapp) already has its `@smoke` tests, which run
against its live site after each deploy (`SMOKE_DIR` in
`sparkhill/webapp_cloudflare/mise.toml`).
