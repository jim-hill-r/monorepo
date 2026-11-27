---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Royalty / Phase 2: honor code and reports"
order: 50
added: 2026-11-27
---

Define `royalty-status.toml`, the producer's public record of income received in the window, and add `royalty status` for a producer to update it from their own ledger. A consumer who runs `royalty verify` reads it, and the calculation applies `[income] on_reach`.
