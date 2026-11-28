---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Cost tracking / Phase 1: measure everything an API shows"
order: 20
added: 2026-11-28
---

Give the daily usage check R2 read, so its "R2 buckets" row is read instead of skipped (cost tracking phase 1, standards/docs/plans/2026-11-cost-tracking.md). The deploy token `jimhillr-deploy` (`CF_DEPLOY_PERMS` in `infra/scripts/mint.sh`) has no Workers R2 Storage Read, so `free-tiers.sh usage` cannot list `r2/buckets` and marks the row skipped; R2 being enabled is an overrun switch the weekly billing check sees only through `subscriptions`. Add "Workers R2 Storage Read" to `CF_DEPLOY_PERMS`, update its `scope` in `standards/docs/threat.toml` and the token note under "Billing checks" in `standards/docs/free-tiers.md`. This is a credential change: label the PR `credential-change`, and Jim approves the Infra Mint run (decisions 0010 and 0011). Check the next daily run shows a number in the R2 row.
