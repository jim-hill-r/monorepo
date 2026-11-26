---
kind: todo
priority: priority
tags: [agent-generated]
added: 2026-11-26
---

Add a `_headers` file to the Pages build with the CSP from `web/src/main.rs` plus `frame-ancestors 'none'` and `Referrer-Policy: no-referrer`, since a `<meta>` CSP can't stop framing. See the [deep review 2026-11-25](../../docs/reviews/2026-11-25-cookbook/deck.md).
