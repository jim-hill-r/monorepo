---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-26
---

`tests/integration_tests.rs` only greps the source files for strings, and
`webapp/src/state.rs` has unit tests only for retries and stabilizing. Test
the rest of `AppState` (bringing stable letters back with
`REINTRODUCE_COUNT`, `consecutive_fails`, the word level) and replace the
string checks with tests of behaviour (standards/docs/testing.md). Decide whether `w` being in both of the
last two letter groups is meant: when it stabilizes, `retain` drops both
copies.
