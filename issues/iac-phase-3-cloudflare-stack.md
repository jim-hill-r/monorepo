---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Infrastructure as code / Phase 3: Cloudflare"
order: 70
added: 2026-11-26
---

Decision 0007, phase 3: a `cloudflare` stack in `infra` that imports every Pages project (wrangler still deploys content) and adds Access for pane, then Workers, D1 and DNS as they appear. Keep `free-tiers.sh pages-project` until IaC creates every Pages project, then let IAC-003 replace it. First add a human issue for a Cloudflare token with Pages and Access edit for apply, and read for plans; token scopes themselves stay a human check.
