---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Feedback on every change
order: 160
added: 2026-11-26
---

Add PostHog session replay to each production web app (production host only, inputs masked, console and network capture on) with a privacy note on each site. The snippet must not load when the user agent contains `jimhillr-evals` (every Playwright, smoke and eval run sends it; eval engine phase 3 in `standards/docs/plans/2026-11-eval-engine.md`), so tests never spend recordings, and a Playwright test should check that.
