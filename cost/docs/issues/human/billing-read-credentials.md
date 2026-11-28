---
added: 2026-11-27
unblocks: "the weekly Billing check for GitHub charges"
runbook: standards/docs/runbooks/billing-read-credentials.md
---

## Requirement

Make a fine-grained GitHub token with only the Plan account permission (read), and store it as the `infra` environment secret `GITHUB_BILLING_READ_TOKEN`. Then the weekly Billing check proves GitHub charged nothing this month. Cloudflare's token was minted by Infra Mint on 2026-11-27 and PostHog's key exists, so both are already checked.

## Why it can't be automated

GitHub's billing usage API for a personal account needs a fine-grained or user token with the Plan permission. The infra GitHub App's installation token can't call it, and no API creates a fine-grained token.

## Runbook

[Make the billing read credentials](../../../../standards/docs/runbooks/billing-read-credentials.md), steps 2 to 4: about 5 minutes, $0.
