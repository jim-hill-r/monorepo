---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Royalty / Phase 3: metered"
order: 40
added: 2026-11-27
---

Build the opt-in in-process counter as its own crate, to the rules in [The in-process counter](../plans/2026-11-royalty-plan.md#the-in-process-counter-if-it-is-ever-built): compiled out without the feature, one per-thread add per counted call, batched off-thread to a local file, fails silent, never enforces. Start with the benchmark gate (at most 2 ns per call and 1% on a tight loop, failing CI when over) and a test that the built output is unchanged with the feature off.

Waits on: the go or no-go at the start of Phase 3 (decision 9, answered: decide then from what producers ask for).
