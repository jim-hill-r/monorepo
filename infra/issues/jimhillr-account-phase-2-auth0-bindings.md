---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "jimhillr account / Phase 2: tenant settings in IaC"
order: 20
added: 2026-11-28
---

Jim accepted [decision 0027](../../standards/docs/decisions/0027-one-jimhillr-account.md) on 2026-11-28. Add `tenant`, `branding`, `prompt`, `prompt_custom_text` and `connection_clients` to `terrars_auth0.json`, regenerate the bindings with the pinned terrars, add the types to `ALLOWED_RESOURCE_TYPES` (none can bill on the free plan) and `auth0_tenant`, `auth0_branding`, `auth0_prompt` and `auth0_connection_clients` to `CREDENTIAL_TYPE_PREFIXES` in `src/guard.rs`, and import the tenant's live settings without changing them, so a read-only plan shows today's friendly name, branding and which connections each client has. [Plan](../../standards/docs/plans/2026-11-jimhillr-account.md#on-the-hosted-page-branding-as-iac).
