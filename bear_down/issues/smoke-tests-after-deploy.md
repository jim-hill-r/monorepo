---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Evals roadmap / Phase 1: Trust the green"
order: 60
added: 2026-11-28
---

Run bear_down's `@smoke` Playwright tests against the live site after each deploy, the way cookbook and sparkhill do: let `bear_down/web/playwright.config.ts` use `BASE_URL` (no dev server, the `jimhillr-evals` user agent), set `SMOKE_DIR = "../web"` in `bear_down/cloudflare/mise.toml`, and list the smoke tests as `cd` `playwright` evals in `evals.toml` ("After deploy" in `../../standards/docs/evals.md`). Today its live checks are `http` requests only. Part of the [evals roadmap](../../evals/docs/roadmap.md).
