---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Native sign-in / Phase 3: Auth0 native app"
order: 30
added: 2026-11-27
---

Add a native Auth0 application for cahokia desktop and mobile to the auth0 stack (`app_type = "native"`, callbacks `http://127.0.0.1:<port>/callback` for each port in auth_sdk's loopback list, `auth_sdk::native::loopback::DEFAULT_PORTS` (53682 to 53685) and the mobile custom scheme, `token_endpoint_auth_method = "none"`, grant `authorization_code` and `refresh_token`). It is a new application, so it's created, not imported, and its client ID (public, not a secret) goes into cahokia's desktop and mobile config. This is a credential change: label the PR `credential-change` and `needs-human`; it applies only after Jim approves its run in `infra-credentials` (decision 0010). First check whether Auth0 now accepts a wildcard loopback port and, if so, use one callback. Plan: auth_sdk/docs/plans/2026-11-native-sign-in.md.
