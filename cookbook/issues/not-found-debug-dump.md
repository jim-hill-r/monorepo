---
kind: fix
priority: backlog
tags: [agent-generated]
added: 2026-11-28
---

The 404 page prints `Attempted to navigate to: [...]`, a debug dump of the route (`PageNotFound` in `web/src/main.rs`). Show a friendly message with links to today's recipe and this week's plan, and label recipe prep, cook and servings with words, not only emoji. Waits for [the main.rs split](split-web-src-main-rs-1.md). See the [deep review](../../docs/reviews/2026-11-25-cookbook/deck.md), "Mobile and accessibility".
