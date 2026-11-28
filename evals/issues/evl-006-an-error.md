---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Evals roadmap / Phase 1: Trust the green"
order: 40
added: 2026-11-28
---

Make EVL-006 (an `http` eval that only checks its path answers) an error, so a green light can't rest on a page that merely loads (recommendation 6 of the [evals review](../../docs/reviews/2026-11-27-evals/deck.md)). First land `evl-006-counts-header-checks.md`, then give each eval still flagged a `body_contains` or `asset_contains` check of real content (on 2026-11-28 that looked like pane's `approve-page-live` and `acknowledge-page-live` and design_system's `live-preview-loads-only-its-own-files`), then fail on EVL-006 in `evals check` and `standards/scripts/evals-check.sh` and update "Checks" in `standards/docs/evals.md`. Part of the [evals roadmap](../docs/roadmap.md).
