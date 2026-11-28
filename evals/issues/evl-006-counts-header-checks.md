---
kind: fix
priority: priority
tags: [agent-generated]
epic: "Evals roadmap / Phase 1: Trust the green"
order: 30
added: 2026-11-28
---

EVL-006 (an `http` eval that checks no content) also flags evals that check a response `header`, such as controlled_chaos's `live-hsts` and `live-nosniff` and sparkhill's `home-screen-icon-live` (`content-type: image/png`, which a single-page app's fallback page can't fake). A header check proves something about the response, so `shallow_http` in `evals/cli/src/check.rs` (EVL-006, around line 243) should treat `header` like `body_contains` and `asset_contains`. Add a test, and update EVL-006's line under "Checks" in `standards/docs/evals.md`. `evals check` showed 9 EVL-006 warnings on 2026-11-28; after this only the evals that really check nothing should be left. Part of the [evals roadmap](../docs/roadmap.md).
