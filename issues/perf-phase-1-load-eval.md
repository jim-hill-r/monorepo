---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Performance / Phase 1: load evals"
order: 10
added: 2026-11-28
---

Add the load eval kind to the evals CLI (PERF-102 to PERF-104 in [performance.md](../standards/docs/performance.md)): run a pinned oha (installed through mise, never curl) at each step of rate for duration against BASE_URL, refuse any BASE_URL that isn't loopback, and check p95_ms, p99_ms, status and max_error_rate from oha's JSON summary. Add a shared perf mise task for Cloudflare Worker projects that builds the Worker, starts wrangler dev --local on 127.0.0.1 with a local D1, runs the load evals and stops it, and a workflow job that runs it for each Worker a PR changes and weekly on main. Start with tofu_state: a requirement for its response time and load evals for its routes, as warnings for a week (PERF-603). See [decision 0032](../standards/docs/decisions/0032-performance-testing.md). Update evals.md with the new kind and the evals project's docs/cost.toml for the runner minutes.
