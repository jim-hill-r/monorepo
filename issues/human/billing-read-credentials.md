---
added: 2026-11-27
unblocks: "the weekly Billing check for Cloudflare and GitHub, which takes them off the 30-day payment method check"
runbook: standards/docs/runbooks/billing-read-credentials.md
---

## Requirement

Approve one Infra Mint run (`what` = `cloudflare`) so it makes `CLOUDFLARE_BILLING_READ_TOKEN` (Billing Read only), and make a fine-grained GitHub token with only the Plan account permission (read), stored as the `infra` environment secret `GITHUB_BILLING_READ_TOKEN`. Then the weekly Billing check proves there is no Cloudflare payment method or paid plan, and no GitHub charge (Jim chose Cloudflare, PostHog and GitHub on 2026-11-27). PostHog's key is part of the PostHog sign-up task.

## Why it can't be automated

Infra Mint runs only after Jim approves them (decision 0010). GitHub's billing usage API for a personal account needs a fine-grained or user token with the Plan permission. The infra GitHub App's installation token can't call it, and no API creates a fine-grained token.

## Runbook

[Make the billing read credentials](../../standards/docs/runbooks/billing-read-credentials.md): about 10 minutes, $0.
