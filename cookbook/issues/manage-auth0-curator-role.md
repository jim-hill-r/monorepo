---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Management app / Phase 1: who can sign in"
order: 10
added: 2026-11-28
---

Add the cookbook management app's Auth0 API, the permission `manage:cookbook`, the role `cookbook-manager` and an Action that requires multi-factor sign-in for anyone holding it, to the `infra` auth0 stack, following item 6 of [decision 0018](../../standards/docs/decisions/0018-feature-flags.md). It is a credential change (decision 0010: label `credential-change`, Jim's second approval). In the same PR, add a human task with its runbook (`write-issue` skill) for Jim to give the role to himself and his wife in the Auth0 dashboard, never in git, and the `standards/settings.toml` entry. See the [roadmap](../docs/roadmap.md).
