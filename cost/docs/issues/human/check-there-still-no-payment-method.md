---
added: 2026-11-26
runbook: standards/docs/runbooks/payment-method-check.md
---

## Requirement

Check what no billing API shows: there is still no payment method on the GitHub, Auth0 and RapidAPI accounts, GitHub's Actions budget is $0 with usage stopped at the limit, Squarespace bills only your existing domain renewals, and Claude Code spend is within $25. Also check any vendor the weekly Billing check shows as "not checked" or failing.

PR #582. Keeps everything on free tiers, see standards/docs/free-tiers.md.

## Why it can't be automated

The weekly Billing check (`standards/scripts/billing-check.sh`) proves what read-only APIs can show. Since 2026-11-27 it checks Cloudflare (no payment method, only $0 subscriptions) and PostHog (free plan, no subscription), so they are off this list. GitHub's charges join once `GITHUB_BILLING_READ_TOKEN` exists (cost/docs/issues/human/billing-read-credentials.md), but that API shows only what was charged, not the card or the budget. The rest has no API: a personal GitHub account's card and Actions budget (the Budgets API is for organizations only), Auth0's plan, RapidAPI billing, Squarespace billing and claude.ai billing. The table is under "Billing checks" in standards/docs/free-tiers.md.

## Runbook

[Check there is no payment method](../../../../standards/docs/runbooks/payment-method-check.md)
