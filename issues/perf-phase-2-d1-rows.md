---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Performance / Phase 2: D1 rows and subrequests"
order: 20
added: 2026-11-28
---

Add a small shared Rust helper for our workers-rs Workers that sums rows_read and rows_written from D1 result metadata and counts fetch subrequests per request, and returns them as Server-Timing (d1 and sub) only when the local-only JIMHILLR_PERF variable is set (PERF-105, PERF-107 in [performance.md](../standards/docs/performance.md)). Teach the load eval max_d1_rows_read, max_d1_rows_written and max_subrequests, read from one probe request. Seed the local D1 from perf/seed.sql at realistic size (PERF-106) and adopt it in tofu_state, telemetry (status 200 or 429 for its rate limiter) and pane, with local stubs for every upstream. Waits on perf-phase-1-load-eval.
