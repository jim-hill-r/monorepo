---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Eval engine
order: 180
added: 2026-11-26
---

Eval engine phase 6 (in-app feedback). Add a `feedback` Rust project with a `feedback/model` crate: `Comment`, `Reply`, `Resolve` and `Anchor` types (URL and route, selector path, element text and role, offset, viewport; or scene, screen and world position and entity for games), the build info, the `feedback.jsonl` line format shared with pane's ratings, and the re-anchoring rules (selector, then text, then position; "moved" when the element is gone), all unit tested.
