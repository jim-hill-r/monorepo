---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Royalty / Phase 1: MVP scanner and report"
order: 80
added: 2026-11-27
---

Make `royalty check` in a consumer's project run the scan and exit 1 when the amount owed is over `[consumer.budget]` with `on_exceed = "fail"`, when an `agreement` or `license` amount cannot fit the budget, or when a dependency's `binding` is not in `[consumer.policy] bindings`. The message names the dependency and the limit it broke. This is the consumer's cost ceiling: a pull request that would break the budget fails before it merges.
