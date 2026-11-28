---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Cost tracking / Phase 3: Auth0"
order: 30
added: 2026-11-27
---

Cost tracking phase 3 (standards/docs/plans/2026-11-cost-tracking.md). Once Jim has set `auth0_m2m_tokens_per_month` in `standards/radar/limits.toml`, add a weekly read of Auth0 monthly active users (`GET /api/v2/stats/active-users`, a `read:stats` token minted by Infra Mint as a credential change) and a count of our own M2M token requests, both checked against the ladder in the plan.
