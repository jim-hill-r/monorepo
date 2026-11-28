# Make the billing read credentials

- **Kind:** one-time, then the GitHub token yearly
- **Status:** open for the GitHub token (`cost/docs/issues/human/billing-read-credentials.md`).
  Step 1 (Cloudflare) was done on 2026-11-27, and the PostHog key exists.
- **Time:** about 10 minutes
- **Cost:** $0. Fine-grained tokens and Cloudflare tokens are free, and the
  weekly check makes three API calls a week.

## What it's for

The weekly **Billing check** (`standards/scripts/billing-check.sh`, the
`billing` job of All Free Tier Usage) proves with read-only credentials
that Cloudflare, GitHub and PostHog still have no payment method, paid plan
or charge ([free-tiers.md](../free-tiers.md#billing-checks)). Until each
credential exists, its vendor shows as "not checked" in the run summary and
stays on your 30-day [payment method check](payment-method-check.md).

## Why a person

- **Cloudflare:** Infra Mint makes the token, but every Infra Mint run
  waits for your approval in `infra-credentials` (decision 0010).
- **GitHub:** GitHub's billing usage API for a personal account takes only
  a fine-grained token or a user token with the "Plan" account permission.
  The infra GitHub App's installation token can't call it, and no API makes
  a fine-grained token, so you make it.
- **PostHog:** its key is made at sign-up, in
  [posthog-sign-up.md](posthog-sign-up.md) step 7, not here.

## Before you start

- Signed in to GitHub as `jimhillr`, admin of `jimhillr/monorepo`.
- This runbook's PR is merged, so Infra Mint knows the new token.

## Steps

1. **Cloudflare.** Go to jimhillr/monorepo > Actions > Infra Mint > Run
   workflow, on `main`, with "what" set to `cloudflare` and "rotate" left
   empty. Approve the run when GitHub asks. It makes only what is missing,
   so its summary lists just `CLOUDFLARE_BILLING_READ_TOKEN` (secret of
   `infra`).
2. **GitHub token.** Open
   [github.com/settings/personal-access-tokens/new](https://github.com/settings/personal-access-tokens/new)
   (Settings > Developer settings > Personal access tokens > Fine-grained
   tokens > Generate new token), then fill in:
   - **Token name:** `jimhillr-billing-read` (a token already made under
     the old name `monorepo-billing-read` keeps working; use the new name
     when you regenerate it)
   - **Expiration:** 1 year (366 days)
   - **Resource owner:** `jimhillr`
   - **Repository access:** Public repositories (read-only). It needs no
     repository at all.
   - **Permissions > Account:** **Plan**, set to Read-only. Leave
     everything else on No access.

   Click Generate token and copy it (it starts with `github_pat_`).
3. **Store it.** jimhillr/monorepo > Settings > Environments > `infra` >
   Environment secrets > Add secret, named `GITHUB_BILLING_READ_TOKEN`.
4. **Run the check.** Actions > All Free Tier Usage > Run workflow, on
   `main`.

## Where secrets go

- `CLOUDFLARE_BILLING_READ_TOKEN`: a secret of `infra`. Infra Mint writes
  it, so nobody sees it.
- `GITHUB_BILLING_READ_TOKEN`: a secret of `infra`. Never a repository
  secret, because PR branches can read those.
- `POSTHOG_BILLING_READ_KEY`: a secret of `infra`
  ([posthog-sign-up.md](posthog-sign-up.md)).

## Verify

The run from step 4 is green, and the **Billing check** table in its
summary shows "Cloudflare: no payment method" and "GitHub: $0 net billing
usage", not "not checked". Tell any thread "done" so it deletes the issue.

## Undo or rotate

- **Cloudflare:** the quarterly Infra Mint run rotates it. To retire it,
  delete `jimhillr-billing-read` in dash.cloudflare.com > Manage Account >
  Account API Tokens.
- **GitHub:** before it expires, the settings check
  `rotate-github-billing-read-token` reminds you. Regenerate it on the same
  page and overwrite the secret. To retire it, delete it there.
