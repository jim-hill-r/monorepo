---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Royalty / Phase 2: honor code and reports"
order: 40
added: 2026-11-27
---

Add `royalty verify` (opt-in, the only command that uses the network): check that each payee's `github` account owns or maintains the package's `repository`, and that the package's manifest names the same repository. Uses the consumer's own read-only GitHub token from the environment; nothing is stored.
