---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Eval engine
added: 2026-11-27
---

Give DSGN-002 (an app theme can't make an app harder to use) a production-stage eval. App themes ship inside each app's wasm (bear_down's `design/theme.css` is `include_str!`d in `bear_down/web/src/main.rs`), so a plain `http` eval can't see them. Once `asset_contains` lands (PR 835), add a `cd` eval with `url = "https://jimhillr-bear-down.pages.dev/"` and `asset_contains = "[data-theme=\"bears\"]"`, so the live Bears theme is the checked one.
