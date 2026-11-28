---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Performance / Phase 4: production metrics"
order: 40
added: 2026-11-28
---

Watch production without loading it (PERF-201 to PERF-205 in [performance.md](../standards/docs/performance.md)): add a read-only Account Analytics token to infra/scripts/mint.sh (a credential change, so it needs Jim's second approval, decision 0010) with its row in secret-rotation.md, a workers-analytics metric source in the evals CLI that reads Workers invocations (cpu_time_p99_ms, duration_p95_ms, error_rate) and D1 analytics (rows read and written a day) from Cloudflare's GraphQL Analytics API, metric evals per Worker and account-wide quota evals in the jimhillr project, and a 500-a-day cap on cd latency probes across the account. Waits on eval-engine-phase-3-metric-eval.
