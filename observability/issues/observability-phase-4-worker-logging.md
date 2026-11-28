---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Observability / Phase 4: Worker logging"
order: 40
added: 2026-11-28
---

Give every Rust Worker one shared `tracing` setup ([decision 0023](../../standards/docs/decisions/0023-observability.md)) that logs one JSON line per request (project, build, route, status, duration) and no personal data, so Workers Logs and Observability can query it for 7 days. Move `telemetry` and `tofu_state` to it.
