---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Monitoring / Phase 1: watchdog Worker"
order: 10
added: 2026-11-28
---

Create the `watchdog` project ([decision 0023](../standards/docs/decisions/0023-monitoring-alerting-logging-paging.md)): a Rust Worker on one Cron Trigger (every 5 minutes) that reads `watchdog/checks.toml` and checks every production URL and Worker (status and, where an eval defines one, a content marker) with the `jimhillr-evals` user agent, and writes results to Analytics Engine. Add an audit check that every project's production `DEPLOY_URL` and every Worker in `standards/resources.toml` has a check. Alerts only log in this phase. Also find out whether Cloudflare Monitors (SQL alerts) are on the Free plan and record the answer in the decision's Revisit section. Update cost.toml, threat.toml, the user guide and the radar exemplar.
