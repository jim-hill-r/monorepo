# Sign up for PostHog

- **Kind:** one-time
- **Status:** done 2026-11-27 (US region; the project token is in
  `cookbook/web/public/posthog.js`, the first site to use it)
- **Time:** about 15 minutes
- **Cost:** $0 on the free plan, which needs no card: 5,000 web recordings
  and 1,000,000 events a month. Past a free limit, PostHog drops the extra
  data instead of billing. Adding a card is what starts a paid
  subscription, so never add one.

Rewritten 2026-11-27 against PostHog's current docs, after Jim found that
the first version didn't match the app. PostHog moves its settings around
often. If a menu path below is wrong, use the search box at the top of
Settings with the name in quotes, and say in the thread which step was off
so an agent can fix it here.

## What it's for

Real-user session replay on production sites and the weekly
`review-user-sessions` task
([decision 0001](../decisions/0001-deploy-feedback-loop.md)). It also lets
the weekly billing check confirm PostHog stays free. PostHog is an approved
exception vendor.

## Why a person

Signing up needs a person to agree to the terms and own the account.
PostHog personal API keys can only be made by a signed-in person: there is
no API that makes the first one.

## Before you start

- An email address to own the account (or sign in with GitHub or Google).
- Pick a region now, since it can't be changed later. **US** (Virginia) is
  the default. **EU** (Frankfurt) keeps visitors' data in Europe.

## Steps

1. **Sign up.** Go to [app.posthog.com/signup](https://app.posthog.com/signup),
   choose the region, and create the account. The free plan needs no card.
   Skip or close any "upgrade", "add billing details" or card step.
2. **Onboarding.** When asked which products to use, pick **Session
   replay** (and Product analytics, which comes with it). If it offers to
   run the setup wizard (`npx @posthog/wizard`) or shows a code snippet,
   skip that. An agent adds the snippet to each site in a PR.
3. **Your project.** Sign-up makes an organization and one project, often
   called "Default Project". The free plan allows one project. You can
   rename it to `jimhillr` in Settings > Project > General, but that's
   optional.
4. **Turn on recording.** Go to Settings > Project > Session replay
   (search "Record user sessions" if it isn't there) and turn on **Record
   user sessions**. On the same page, under privacy or masking, keep
   **mask all inputs** on, which is the default. Turn on masking all text
   too, if the page offers it. The site snippet also masks inputs and
   passwords, so recordings never hold what visitors type.
5. **Copy the project token.** It's in Settings > Project > General, under
   **Project token** (older pages call it the "Project API key"). It starts
   with `phc_`. It's public and ingest-only, so it goes in each site's
   config, not in a secret. Note it and the region for the reply in step 9.
6. **Make the replay key.** Open
   [Personal API keys](https://us.posthog.com/settings/user-api-keys) (EU:
   `eu.posthog.com/settings/user-api-keys`). It's also under your avatar >
   Account settings > Personal API keys. Click **Create personal API key**,
   then fill in:
   - **Label:** `monorepo-replay-read`
   - **Access:** only this project, if it lets you pick
   - **Scopes:** **Read** for `session_recording`, `query` and `project`,
     and no access to anything else. Pick them one by one rather than
     using a preset, since a preset may grant more.

   Copy the key (it starts with `phx_`). PostHog shows it only once.
7. **Make the billing key.** Click Create again, then fill in:
   - **Label:** `monorepo-billing-read`
   - **Access:** the organization
   - **Scopes:** **Read** for `billing` only

   This one is kept apart because `billing:read` covers the whole
   organization's billing. It goes in the `infra` environment, which only
   `main` can use, so no PR branch ever sees it.
8. **Store the keys** as the secrets below, in jimhillr/monorepo >
   Settings > Secrets and variables > Actions. If you chose EU, also add a
   repository **variable** (Variables tab, not Secrets) `POSTHOG_API_HOST`
   with the value `https://eu.posthog.com`. US needs no variable.
9. **Check billing.** Settings > Organization > Billing (or
   [app.posthog.com/organization/billing](https://app.posthog.com/organization/billing))
   shows the free plan and no active subscription. Don't add a card or a
   billing limit: with no card nothing can be charged, and the free plan
   drops data past its limits. Then reply in the Claude project with
   "done", the project token and the region.

## Where secrets go

- `POSTHOG_PERSONAL_API_KEY`: the replay key from step 6. A repository
  secret (Settings > Secrets and variables > Actions > Repository secrets),
  for the weekly session review.
- `POSTHOG_BILLING_READ_KEY`: the billing key from step 7. A secret of the
  `infra` environment (Settings > Environments > infra > Environment
  secrets), for the weekly billing check.
- The project token isn't a secret: reply with it and an agent adds it to
  the sites.

## Verify

- Both secrets show in the GitHub settings pages above. GitHub never shows
  their values.
- After an agent adds the snippet to a production site and you visit it,
  PostHog's Session replay page shows your visit.
- Once the billing check is live, the weekly **All Free Tier Usage** run's
  summary shows "PostHog: no active subscription" instead of "not
  checked".

## Undo or rotate

- **Rotate a key:** in Personal API keys, use **Roll** on the key (or
  delete it and make a new one with the same scopes). Then overwrite its
  secret in GitHub. `standards/docs/secret-rotation.md` lists the cadence.
- **Stop recording:** turn off "Record user sessions" in step 4. Deleting
  the project deletes every recording.
