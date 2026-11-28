---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-26
---

Add one Auth0 helper that builds the issuer (with its trailing slash), authorize and token URLs from a tenant domain, and use it in cahokia and cookbook so their Auth0 config can't drift. See `cahokia/docs/plans/2026-11-prod-oidc-error.md`, step 4.
