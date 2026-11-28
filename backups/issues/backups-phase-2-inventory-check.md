---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Backups / Phase 2: drills"
order: 30
added: 2026-11-28
---

Add an audit check (a `chores` command, BACK-004): every D1 database and KV namespace named in a `wrangler.toml` or the `cloudflare` IaC stack has a row with a tier in backups/docs/inventory.md. Warn first, then fail once the inventory is complete. Add the rule to standards/docs/resources.md.
