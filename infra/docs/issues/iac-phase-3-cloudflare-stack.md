---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Infrastructure as code / Phase 3: Cloudflare"
order: 70
added: 2026-11-26
---

Decision 0007, phase 3: the `cloudflare` stack in `infra` (started early with Web Analytics on each production site, Jim's choice on 2026-11-26) also imports every Pages project (wrangler still deploys content) and adds Access for pane, then Workers, D1 and DNS as they appear. Keep `free-tiers.sh pages-project` until IaC creates every Pages project, then let IAC-003 replace it. Add Pages and Access to the two tokens in `docs/issues/human/cloudflare-iac-credentials.md` (edit for apply, read for plans); token scopes themselves stay a human check.
