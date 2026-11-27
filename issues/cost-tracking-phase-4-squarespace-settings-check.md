---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Cost tracking / Phase 4: Squarespace domains"
order: 60
added: 2026-11-27
---

Add a `squarespace-domains` entry to `standards/settings.toml` (vendor Squarespace, account.squarespace.com > Domains, every 90d) expecting that the domains, prices, renewal dates and auto-renew settings match the Squarespace `[[cost]]` entries, and point Squarespace's billing entry in `radar.json` at it. Before that, check once more whether Squarespace now offers a read-only API for domains or billing. If it does, propose an API check in `standards/scripts/billing-check.sh` instead, as a decision with its cost.
