---
added: 2026-11-26
runbook: standards/docs/runbooks/payment-method-check.md
---

## Requirement

Check there is still no payment method on the Cloudflare, PostHog, Auth0 and RapidAPI accounts, and that GitHub's Actions budget is $0 with usage stopped at the limit.

PR #582. Keeps everything on free tiers, see standards/docs/free-tiers.md.

## Why it can't be automated

These vendors' billing pages have no read API an agent can use, and agents must never hold a token with billing access.

## Runbook

[Check there is no payment method](../../standards/docs/runbooks/payment-method-check.md)
