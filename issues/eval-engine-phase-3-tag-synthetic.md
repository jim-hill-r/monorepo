---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Eval engine
order: 110
added: 2026-11-26
---

Eval engine phase 3. Tag synthetic traffic: every smoke, eval and chaos run (Playwright configs, `standards/scripts/evals.sh` HTTP checks) sends the `jimhillr-evals` user agent, and non-browser runs also send `x-jimhillr-synthetic: 1`; the PostHog snippet doesn't load for that user agent. The `telemetry` Worker already drops events with either one.
