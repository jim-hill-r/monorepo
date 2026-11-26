---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Eval engine
order: 200
added: 2026-11-26
---

Eval engine phase 6. Add `feedback/toolbar`, a framework-free Rust wasm crate (`web-sys`) loaded by one `<script type="module">` tag: hidden unless `?feedback` or the hotkey turns it on, comment mode with element outline and numbered pins, kind (bug, idea, not met, looks good) and requirement picker, viewport screenshot with the pin drawn (at most 1280 px and 200 KB, WebP), last 20 console errors, sign-in through a pane popup that returns a 12-hour token by `postMessage`, sending to pane's `/api/feedback`, and showing the page's open pins and replies. Add it to cookbook/web first with Playwright tests.
