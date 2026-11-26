---
kind: todo
priority: priority
tags: [agent-generated]
added: 2026-11-26
---

Add a `[profile.release]` to `cookbook/Cargo.toml` (`opt-level = "z"`, `lto = true`, `codegen-units = 1`, `strip = true`) and record the wasm size before and after (3,432,336 bytes on 2026-11-25). See the [deep review 2026-11-25](../../docs/reviews/2026-11-25-cookbook/deck.md).
