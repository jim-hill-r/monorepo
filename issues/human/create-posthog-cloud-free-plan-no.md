---
added: 2026-11-25
unblocks: "PostHog session replay and the `review-user-sessions` skill, decision 0001"
runbook: standards/docs/runbooks/posthog-sign-up.md
---

## Requirement

Create the PostHog Cloud project (free plan, no card), turn on session replay, and make two read-only personal API keys: `POSTHOG_PERSONAL_API_KEY` (session recording, query and project read) as a repository secret, and `POSTHOG_BILLING_READ_KEY` (billing read only) as a secret of the `infra` environment, for the weekly billing check. Reply with the project token and region. The runbook was rewritten on 2026-11-27 to match PostHog's current app.

## Why it can't be automated

Signing up needs a person to agree to the terms and own the account, and PostHog personal API keys can only be made by a signed-in person.

## Runbook

[Sign up for PostHog](../../standards/docs/runbooks/posthog-sign-up.md)
