---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Cost tracking / Phase 4: Squarespace domains"
order: 60
added: 2026-11-27
---

Once the Squarespace domains are recorded as `[[cost]]` entries, extend the `squarespace-billing` settings check in `standards/settings.toml` (added by the billing check work, every 90d) so that its `expect` also covers the following: each domain's price, renewal date and auto-renew setting match its `[[cost]]`. Don't add a second setting. Before that, check once more whether Squarespace now offers a read-only API for domains or billing. If it does, propose an API check in `standards/scripts/billing-check.sh` instead, as a decision that gives its cost.
