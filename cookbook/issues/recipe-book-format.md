---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Content curation / Phase 1: a book-ready model"
order: 10
added: 2026-11-28
---

Write `docs/content-format.md`, the recipe and plan format the book is written in, and add `Status` (`placeholder`, `draft`, `tested`, `book-ready`), `Yield`, `Source` and `Equipment` to `Recipe` in `core` and to the parser and writer in `data_md`, with round-trip tests. Mark all 365 generated recipes `Status: placeholder`, and fail CI on an unknown status. Don't touch `web/src/main.rs` (it is being split). See the [roadmap](../docs/roadmap.md), "Now: curation foundations".
