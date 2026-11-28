---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "User feedback / Phase 8: one feedback tool with roles"
order: 81
added: 2026-11-26
---

Add the staff modes to the one Feedback button (`feedback/toolbar`, framework-free Rust wasm with `web-sys`): a staff sign-in (Auth0, as decision 0018's management apps use) returns a role, app manager, product owner or engineering, and only then loads the staff code. Staff modes add comment mode with element outline and numbered pins, kind (bug, idea, not met, looks good) and requirement picker, viewport screenshot with the pin drawn (at most 1280 px and 200 KB, WebP), last 20 console errors and the build, and show the page's open pins and replies. App manager feedback goes to the product's own inbox; product owner ratings to `<project>/evals/feedback.jsonl` through pane; engineering feedback becomes FIX or TODO issue files. Signed out, the button stays the privacy-safe user form. Add it to cookbook/web first with Playwright tests. Moved from eval engine phase 6. See [the plan](../plans/2026-11-user-feedback.md#one-feedback-tool-four-roles).
