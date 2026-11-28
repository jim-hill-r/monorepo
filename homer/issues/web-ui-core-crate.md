---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Web UI / Phase 1: shared core"
order: 10
added: 2026-11-28
---

Move scoring and the report out of the CLI into a `homer_core` crate (`homer/core`) with no network or file code, so the web app can compile it to WebAssembly: `HomePreferences`, `PropertyEvaluation`, `evaluate_property`, `parse_property_details` (Zillow JSON in, `PropertyDetails` out), the report and JSON formatters, and the quota rules in `src/quota.rs` rewritten to take plain header values instead of a `reqwest::HeaderMap`. Turn `homer` into a small workspace like `cookbook` (`core`, `cli`; package names per the naming standard), keep every CLI flag, output and exit code the same, and keep the existing unit and trycmd tests passing unchanged as the proof. Check `cargo build -p homer_core --target wasm32-unknown-unknown` in CI. Land after the open PRs 921 and 962, or rebase on them. See the [roadmap](../docs/roadmap.md) and [decision 0001](../docs/decisions/0001-web-ui-data-source.md).
