---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-26
---

Prices print as raw numbers (`Price: $750000`, `Price: $900000 (max is $800000)`), which are hard to read at a glance. Format them with thousands separators (`$750,000`) in `format_property` and in the unmet-criteria messages of `evaluate_property` (`../src/main.rs`), with a small helper and unit tests (0, 999, 1000, a large value). Update the trycmd transcripts with `TRYCMD=overwrite cargo test --test cli` and the example in `../README.md`.
