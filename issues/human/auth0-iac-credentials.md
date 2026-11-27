---
added: 2026-11-26
unblocks: "infrastructure as code phase 1 (decision 0007), which then manages the Auth0 callback URLs instead of the dashboard"
runbook: standards/docs/runbooks/auth0-iac-credentials.md
---

## Requirement

In the Auth0 dashboard, create two Machine to Machine applications for the Auth0 Management API: "infra-read" with `read:clients`, and "infra-apply" with `read:clients` and `update:clients`. Add their domain, client ID and client secret as GitHub secrets: the read one as repository secrets `AUTH0_INFRA_READ_*`, the apply one in a new `infra` environment limited to `main` as `AUTH0_INFRA_APPLY_*`. Also add `TOFU_STATE_PASSPHRASE` (a long random string) to the `infra` environment. The read app is also the read-only token the resource cleanup plan's phase 4 needs.

Also raise the `auth0_m2m_tokens_per_month` budget in `standards/radar/limits.toml` from 0 (set when CI used only mocks) to about 60: the PR plans, applies and the weekly drift plan each get one token, about 30 to 60 a month against the free limit of 1,000 (decision 0007). Only you set budgets, so say the number in a reply or PR and an agent makes the edit.

## Why it can't be automated

These are the first Auth0 credentials automation gets, so nothing can make them before they exist, and a client secret made by IaC would land in OpenTofu state, which IAC-002 forbids. Creating the `infra` environment needs repo Admin, which agents don't have. The token-minting workflow (Jim, 2026-11-27) is expected to replace this task.

## Runbook

[Create the Auth0 IaC applications](../../standards/docs/runbooks/auth0-iac-credentials.md)
