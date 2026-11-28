---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Evals roadmap / Phase 1: Trust the green"
order: 70
added: 2026-11-28
---

Run pane's `@smoke` Playwright tests (`pane/web/tests/smoke.spec.ts`) against the live site after each deploy: set `SMOKE_DIR = "../web"` in `pane/cloudflare/mise.toml` (its `playwright.config.ts` already reads `BASE_URL`) and list the smoke tests as `cd` `playwright` evals in `evals.toml`, the way cookbook does ("After deploy" in `../../standards/docs/evals.md`). If pane is behind Cloudflare Access by then, the run needs the Access service token from the `infra` mint, never a person's sign-in. Part of the [evals roadmap](../../evals/docs/roadmap.md).
