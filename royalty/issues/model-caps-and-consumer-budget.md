---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Royalty / Phase 1: MVP scanner and report"
order: 30
added: 2026-11-27
---

In `royalty_model`, apply the ceilings to priced amounts: the producer's `[caps]` (project, then person or organization with the consumer cap as fallback), the consumer's `[consumer.budget]` (per producer, then total) with `on_exceed` `fail` or `scale`, the rule that only `voluntary` amounts are ever scaled, `[[consumer.package]]` exceptions, and `minimum_payment_cents`. Steps 6, 8 and 9 of [How the amount is worked out](../docs/royalty-toml.md#how-the-amount-is-worked-out) and [Exact rules](../docs/royalty-toml.md#exact-rules), for a single period and a single project (year-to-date and merging come in Phase 2). Each result records which cap or budget set the final number. A test reproduces the ["What Acme owes in one month"](../docs/royalty-toml.md#what-acme-owes-in-one-month) table.
