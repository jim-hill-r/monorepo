---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

`src/main.rs` is about 1,400 lines holding the CLI, the Zillow client, JSON parsing, evaluation, text and JSON formatting and all their tests, and its `pub` items are not reachable from anywhere because homer is a binary-only crate. Add `src/lib.rs` and move the code into `zillow.rs` (client and `parse_property_details`), `evaluate.rs` (preferences and `evaluate_property`) and `report.rs` (`format_property`, `format_json`, `format_dollars`), leaving `main.rs` with `Args` and `main` only. Keep the tests next to the code they test, and document each module per `standards/docs/rust.md`.
