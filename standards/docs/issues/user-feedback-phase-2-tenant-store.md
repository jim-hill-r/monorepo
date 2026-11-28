---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "User feedback / Phase 2: tenant store"
order: 20
added: 2026-11-28
---

Build [decision 0021](../decisions/0021-tenant-scoped-product-data.md): a technical-platform `tenant_store` project (workers-rs, d1 feature) whose `TenantStore::open(env)` takes the tenant from the Worker's config, binds it in every query (a test fails any query without it) and refuses writes past the tenant's daily budget; the `jimhillr-products` D1 database in the infra cloudflare stack; per-tenant budgets in standards/radar/limits.toml and free-tiers.sh; and `chores tenant-export <tenant>` writing one tenant's rows as a .sql file. Covers FDBK-011.
