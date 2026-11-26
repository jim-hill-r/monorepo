---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Eval engine
order: 110
added: 2026-11-26
---

Eval engine phase 3. Tag synthetic traffic: every smoke, eval and chaos run sends the `monorepo-evals` user agent and `x-monorepo-synthetic: 1`; the PostHog snippet doesn't load for that user agent; the `telemetry` Worker marks those events `synthetic`.
