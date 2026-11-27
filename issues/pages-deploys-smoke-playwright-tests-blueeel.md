---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Feedback on every change
order: 70
added: 2026-11-26
---

Add `@smoke` Playwright tests for sparkhill/webapp (deployed by
`sparkhill/webapp_cloudflare`), and a Pages deploy and `@smoke` tests for
blueeel, matching cookbook, pane and cahokia.

For Blue Eel, tag the `@smoke` tests with the `@req:SPRK-00n` they check
and add `cd` to those Playwright evals in `sparkhill/evals.toml`: today the
production evals of SPRK-002 to SPRK-005 only check that the app shell
answers, so tracing, levels and saved progress are checked in production
only by the weekly judge.
