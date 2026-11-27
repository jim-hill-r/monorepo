---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Royalty / Phase 2: honor code and reports"
order: 10
added: 2026-11-27
---

Add `royalty.lock`: for each producer family, the version, a hash of its terms and its payees as the consumer last accepted them. `royalty check` fails when a dependency update changes a payee or raises the amount owed by more than `[consumer.policy] review_above_cents`, and prints the difference; `royalty accept` records the new terms. This is the main defence against a hijacked payee.
