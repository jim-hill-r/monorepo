---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Feature flags / Phase 2: IaC and Auth0"
order: 20
added: 2026-11-27
---

Add the `jimhillr-flags` D1 database and Worker to the infra cloudflare stack, and the Auth0 API, `manage:{app}` permissions, `{app}-manager` roles and the multi-factor Action for managers to the auth0 stack (a credential change: Jim's second approval, decision 0010). Add the catalog token to Infra Mint with its secret-rotation row, a human task and runbook for appointing an app manager in Auth0, and the `standards/settings.toml` entries. Confirm which MFA factors Auth0's free plan offers. See [decision 0018](../../../standards/docs/decisions/0018-feature-flags.md) item 6.
