---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Resource cleanup / Phase 4: Auth0 and DNS"
order: 10
added: 2026-11-26
---

Resource cleanup phase 4 (see the [plan](../plans/2026-11-resource-cleanup.md)). Once the IaC work's read-only Auth0 application exists (`AUTH0_INFRA_READ_*` secrets, made by the Infra Mint workflow, decision 0011), add a weekly read of every Auth0 application's callback, origin and logout URLs to the sweep, flag any URL no `threat.toml` claims as an orphan, and let `all-retire-resources.yml` check Auth0 references itself instead of waiting on a human issue. Needs threat check phase 2's `[auth] callbacks`.
