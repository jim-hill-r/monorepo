---
added: 2026-11-26
runbook: standards/docs/runbooks/payment-method-check.md
---

## Requirement

Check there is still no payment method on the GitHub, Auth0 and RapidAPI accounts, that Squarespace bills only your existing domain renewals, that GitHub's Actions budget is $0 with usage stopped at the limit, and that Claude Code spend is within $25. Check Cloudflare and PostHog only while the weekly Billing check shows them as "not checked".

PR #582. Keeps everything on free tiers, see standards/docs/free-tiers.md.

## Why it can't be automated

The weekly Billing check (`standards/scripts/billing-check.sh`) now covers what read-only APIs can show: Cloudflare's payment methods and subscriptions, GitHub's monthly charges and PostHog's subscription. These have no API: a personal GitHub account's card and Actions budget (the Budgets API is for organizations only), Auth0's plan, RapidAPI billing, Squarespace billing and claude.ai billing. Every vendor in the tech radar has a `billing` entry saying which applies (RAD-007). The table is under "Billing checks" in standards/docs/free-tiers.md.

## Runbook

[Check there is no payment method](../../standards/docs/runbooks/payment-method-check.md)
