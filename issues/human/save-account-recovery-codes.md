---
added: 2026-11-28
unblocks: recovering any account after a lost phone or a lockout (decision 0030)
runbook: standards/docs/runbooks/account-recovery-codes.md
---

## Requirement

Save the 2FA recovery codes, or register a second passkey, for every account the repo stands on: GitHub (jimhillr, jimhillr-agt, jimhillr-reviewer), Cloudflare, Auth0, PostHog, Squarespace and Claude. Keep them in your password manager and on paper somewhere safe at home. Losing one of these accounts is the worst disaster in decision 0030, and no backup can bring it back.

## Why it can't be automated

Recovery codes are shown only to the signed-in account owner, and must be kept somewhere no agent, workflow or vendor account can reach, or they don't survive losing that account.

## Runbook

[Save the account recovery codes offline](../../standards/docs/runbooks/account-recovery-codes.md)
