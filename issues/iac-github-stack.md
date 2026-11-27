---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Infrastructure as code / Phase 2: GitHub"
order: 10
added: 2026-11-27
---

Add a `github` stack to `infra` (decision 0007) that manages this repository's settings with the integrations/github provider. The provider signs in as the infra GitHub App ([runbook](../standards/docs/runbooks/github-app-infra.md)), with a token minted per run by `actions/create-github-app-token`, in `infra-credentials` only, since every `github_*` change is a credential change (decision 0010).

Start with the two settings that are human tasks today:
- `github-settings-code-security-turn-secret.md`: secret scanning with push protection, Dependabot alerts and security updates, private vulnerability reporting.
- `github-settings-rules-rulesets-default-ruleset.md`: require the `CI result` check on `main`.

Never manage a ruleset's bypass list or the `infra-credentials` environment (IAC-005). Import what exists, allowlist each new type in `src/guard.rs`, and delete those two human issues in the PR that applies them.
