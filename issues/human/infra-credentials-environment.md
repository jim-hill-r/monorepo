---
added: 2026-11-26
unblocks: "every infra workflow: the Infra Mint workflow makes all other infra tokens and passwords from these, then the state Worker deploys and the plan, apply and drift workflows run (decisions 0007, 0009, 0010)"
runbook: standards/docs/runbooks/infra-credentials-environment.md
---

## Requirement

Create the `infra` and `infra-credentials` GitHub environments and the three bootstrap credentials, all stored only in `infra-credentials`:
- the infra GitHub App (`github-app-infra-credentials.md`)
- an Auth0 Machine to Machine app that can create apps and grants
- a Cloudflare account token that can create tokens

Then approve one run of the Infra Mint workflow, and set the Auth0 token budget. After that, the workflow makes every other infra secret (state passwords, the deploy, read and apply tokens, the Auth0 apps, the Android key) and nobody makes or sees them (Jim, 2026-11-27). This replaces the old separate tasks for the state passwords, the origin variable, the Auth0 and Cloudflare infra tokens, the deploy token's scopes, Analytics read and the Android key.

## Why it can't be automated

These are the first credentials everything else is minted from, so no automation can exist before them. Making a vendor token without one needs a signed-in person. They also have to be yours, behind your approval on `infra-credentials`, so that an agent can never widen its own access (decision 0010). Only you set budgets.

## Runbook

[standards/docs/runbooks/infra-credentials-environment.md](../../standards/docs/runbooks/infra-credentials-environment.md): about 30 minutes, $0.
