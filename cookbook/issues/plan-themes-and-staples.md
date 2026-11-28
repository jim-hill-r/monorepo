---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Content curation / Phase 1: a book-ready model"
order: 20
added: 2026-11-28
---

Add a free-text `Theme` and a `Staples` list to `Plan` (COOK-008) in `core` and `data_md`, fill them in the 53 plan files as placeholders, and show both on the plan page above the shopping list. Add a test that every plan file parses with a theme, and an eval for COOK-008 in `evals.toml`. The web change waits for [the main.rs split](split-web-src-main-rs-1.md) to merge. See the [roadmap](../docs/roadmap.md).
