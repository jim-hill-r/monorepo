# Sign up for PostHog

- **Kind:** one-time
- **Status:** open (`issues/human/create-posthog-cloud-free-plan-no.md`)
- **Time:** about 10 minutes
- **Cost:** $0 on the free plan with no card: 5,000 web recordings and
  1,000,000 events a month. With no card on file it can't bill; adding one
  turns on pay-as-you-go, so never add one.

## What it's for

Real-user session replay on production sites and the weekly
`review-user-sessions` task
([decision 0001](../decisions/0001-deploy-feedback-loop.md)). PostHog is an
approved exception vendor.

## Why a person

Signing up needs a person to agree to the terms and own the account, and
PostHog personal API keys can only be made by a signed-in person.

## Before you start

- An email address to own the account.

## Steps

1. Go to posthog.com > Get started, and sign up. Pick PostHog Cloud (US
   or EU; EU keeps visitors' data in Europe). Skip any billing or card
   step: stay on the free plan.
2. Create one project, for example `jimhillr`.
3. In the project's Settings > Project > Replay, turn on session
   recording and "Mask all text inputs".
4. Copy the project API key (it starts `phc_`). It is public and
   ingest-only, so it goes in each app's config, not in secrets. Reply with
   it, and the region, and an agent adds it.
5. Your avatar > Account settings > Personal API keys > Create. Scope:
   this project only, read access to Session recording, Query and Project.
   Copy it (it starts `phx_`).
6. Add it as the repository secret below.
7. Settings > Billing: check no card is on file.

## Where secrets go

- `POSTHOG_PERSONAL_API_KEY`: a repository secret (read-only key for the
  weekly review). The project key is not a secret.

## Verify

Reply "done" with the project key and region. After an agent adds the
snippet to a production site, PostHog's Replay page shows a recording.
`check-there-still-no-payment-method` covers billing every 30 days.

## Undo or rotate

Personal API keys > Roll or Delete, then overwrite the secret. Deleting
the project in Settings stops all recording.
