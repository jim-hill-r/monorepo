---
kind: fix
priority: priority
tags: [agent-generated]
added: 2026-11-28
---

The Approve page never lists a PR that is waiting for a human in the `Human review` check, which are exactly the PRs only Jim can approve. `READY_SEARCH_URL` in `pane/model/src/queue.rs` asks GitHub search for `status:success`, and a held PR has a pending `Human review` commit status (a failing check run before 2026-11-28), so it counts under "waiting" instead. List a non-draft PR whose only non-green status is a pending `Human review` (for example with a second search, `label:human-review review:required`, checked against `PULLS_URL`; the gate labels held PRs `human-review`) in the `decide` group, with its cost or red attention badge, and add a unit test in `queue.rs` and a case in `pane/web/tests/approve.spec.ts`. See "Human review" in `standards/docs/pull-requests.md`.
