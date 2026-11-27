---
kind: todo
priority: priority
tags: [agent-generated]
epic: Feedback on every change
order: 20
added: 2026-11-26
---

Smoke UAT for cahokia/web. Tag a few Playwright tests `@smoke`, let `playwright.config.ts` use `BASE_URL` (no dev server, synthetic user agent, video, screenshots and traces), and set `SMOKE_DIR = "../web"` in cahokia/cloudflare's `mise.toml` so the deploy job runs them, the way cookbook does (cookbook/web/playwright.config.ts, `standards/scripts/evals.sh smoke`, "After deploy" in `standards/docs/evals.md`). Sign-in can't be tested against production without a test account, so tag only signed-out tests.
