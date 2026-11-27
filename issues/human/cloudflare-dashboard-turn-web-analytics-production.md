---
added: 2026-11-25
unblocks: the telemetry issues in the root issues/
runbook: standards/docs/runbooks/cloudflare-deploy-token.md
---

## Requirement

In the Cloudflare dashboard (My Profile > API Tokens), give the `CLOUDFLARE_API_TOKEN` secret's token "Account Analytics: Read", so it can query Analytics Engine. About 2 minutes, $0; it can be done in the same edit as `cloudflare-api-token-workers-d1-edit.md`.

Web Analytics for each production Pages project is no longer a dashboard step: the `cloudflare` stack in `infra` turns it on (`cloudflare_web_analytics_site`, Jim chose IaC on 2026-11-26), once `cloudflare-iac-credentials.md` and the plan and apply workflows are done.

## Why it can't be automated

Editing a token's permissions needs a token that can edit tokens (API Tokens: Edit), which no agent has. The token-minting workflow (Jim, 2026-11-27) is expected to replace this task.

## Runbook

[Change the Cloudflare deploy token's permissions](../../standards/docs/runbooks/cloudflare-deploy-token.md)
