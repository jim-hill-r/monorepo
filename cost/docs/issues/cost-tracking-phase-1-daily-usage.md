---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Cost tracking / Phase 1: measure everything an API shows"
order: 10
added: 2026-11-27
---

Cost tracking phase 1 (standards/docs/plans/2026-11-cost-tracking.md). Run the free-tier usage check daily on its own workflow (keep the full-history secret scan weekly), write `usage.json` as the run's artifact (shared with `pane/docs/issues/give-free-tiers-sh-usage-json.md`), apply the green, yellow and red ladder from the plan, and add live reads for repo visibility, Cron Triggers, D1 databases and size, R2 buckets, and the busiest day's Workers requests and D1 rows from Cloudflare's GraphQL analytics API. The analytics read needs Account Analytics read on the usage token, added through Infra Mint (`infra/scripts/mint.sh`) as a credential change. Coordinate with the billing checks in the same workflow file.
