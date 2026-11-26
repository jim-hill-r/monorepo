---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Feedback on every change
order: 190
added: 2026-11-26
---

Add a `telemetry` Rust Worker project with a `POST /event` route that validates and rate-limits events (`error`, `game`, `cli`, `deploy`) and writes them to an Analytics Engine dataset and Workers Logs.
