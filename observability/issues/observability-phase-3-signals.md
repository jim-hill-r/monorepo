---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Observability / Phase 3: signals"
order: 30
added: 2026-11-28
---

Add the watchdog's other rules ([decision 0023](../../standards/docs/decisions/0023-observability.md)): errors per project five times higher after its latest `deploy` event (Analytics Engine), free-tier stops that would hit today (high urgency), an hourly GitHub Actions job that raises an alert when the watchdog has not run for 15 minutes, low-urgency alerts from nightly CD, Infra apply and production eval failures, and Cloudflare Notifications (Pages deployment failed and other free ones) through IaC with a webhook to the watchdog's `/notify` route, checked against a secret Infra Mint makes.
