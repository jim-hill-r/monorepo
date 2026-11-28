---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Performance / Phase 3: timed flows"
order: 30
added: 2026-11-28
---

Time key interactions in Playwright (PERF-301, PERF-302 in [performance.md](../standards/docs/performance.md)): the app marks start and end with performance.mark, a test tagged @perf and @req:<ID> reads performance.measure and asserts the requirement's time, once normally and once with 4x CPU throttling and a slow network. Start with the cookbook's search, adding a requirement with a time to cookbook/docs/requirements.md (a requirement change, so it needs Jim), and tag the test @smoke so it also runs after deploy. Then file an issue per product with a time-bound flow (bear_down, cahokia, pane, sparkhill/webapp).
