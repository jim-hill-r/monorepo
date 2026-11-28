---
kind: fix
priority: priority
tags: [agent-generated]
added: 2026-11-28
---

The Approve page doesn't flag PRs labelled `human-review` (a red attention tag or a cost change, waiting only for a human's approval; "Human review" in `standards/docs/pull-requests.md`). Add `human-review` to the labels `pane/model/src/queue.rs` shows, as its own badge in the approve group (it needs Jim's approval, not a decision), with a unit test in `queue.rs` and a case in `pane/web/tests/approve.spec.ts`. Before 2026-11-28 these PRs carried a pending or failing `Human review` status and were left out of the ready search entirely.
