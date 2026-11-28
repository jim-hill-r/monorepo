---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Observability / Phase 1: watchdog Worker"
order: 10
added: 2026-11-28
---

Build the watchdog in the `observability` project ([decision 0023](../../standards/docs/decisions/0023-observability.md)): a Rust Worker on one Cron Trigger (every 5 minutes) that reads `observability/checks.toml` and checks every production URL and Worker (status and, where an eval defines one, a content marker) with the `jimhillr-evals` user agent, and writes results to Analytics Engine. Add an audit check that every project's production `DEPLOY_URL` and every Worker in `standards/resources.toml` has a check. Alerts only log in this phase. Also find out whether Cloudflare Monitors (SQL alerts) are on the Free plan and record the answer in the decision's Revisit section. Turn `observability` from a docs project into a Rust Worker project, and update its cost.toml, threat.toml and user guide and the radar exemplar.
