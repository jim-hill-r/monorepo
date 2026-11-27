---
added: 2026-11-27
unblocks: "the telemetry Worker's first deploy, so bear_down's fan votes and A/B exposures are stored (fan feedback plan, step 2)"
runbook: standards/docs/runbooks/cloudflare-analytics-engine.md
---

## Requirement

Turn on Workers Analytics Engine once for the Cloudflare account, in the dashboard, then re-run the failed `ci (telemetry)` job of the Push Main CI/CD run (or tell an agent to). Until then every telemetry deploy fails with "You need to enable Analytics Engine" (code 10089). Cost: $0, about 2 minutes; the free tier allows 100,000 data points a day.

## Why it can't be automated

Cloudflare has no API to turn Analytics Engine on: its API only has the SQL query endpoints, and wrangler asks for the dashboard. So neither Infra Mint nor IaC can do it.

## Runbook

[Turn on Workers Analytics Engine](../../standards/docs/runbooks/cloudflare-analytics-engine.md)
