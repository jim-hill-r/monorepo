# Change the Cloudflare deploy token's permissions

- **Kind:** on demand (when a deploy or check needs a new permission)
- **Status:** open for Workers Scripts: Edit and D1: Edit
  (`issues/human/cloudflare-api-token-workers-d1-edit.md`) and Account
  Analytics: Read (`issues/human/cloudflare-dashboard-turn-web-analytics-production.md`).
  Do both in one edit.
- **Time:** about 2 minutes
- **Cost:** $0. Deploys stay within the Workers and D1 free tiers
  (`standards/scripts/free-tiers.sh` checks before each one).

## What it's for

`CLOUDFLARE_API_TOKEN` is the token every Cloudflare deploy in
`push-main.yml`, `nightly-cd.yml` and `tofu-state-deploy.yml` uses. Today
it has Cloudflare Pages: Edit. The `tofu_state` Worker deploy needs Workers
Scripts: Edit and D1: Edit (Edit also lets the weekly free-tier usage run
count Workers), and the telemetry review needs Account Analytics: Read to
query Analytics Engine.

## Why a person

Editing a token's permissions needs a token that can edit tokens (API
Tokens: Edit), which no agent has and none should.

## Before you start

- Signed in to dash.cloudflare.com with access to the account.

## Steps

1. dash.cloudflare.com > My Profile > API Tokens. Find the token stored in
   `CLOUDFLARE_API_TOKEN` (its name is on the list; if unsure, it's the one
   with Cloudflare Pages: Edit).
2. ... > Edit. Under Permissions add, each for Account:
   - Workers Scripts > Edit
   - D1 > Edit
   - Account Analytics > Read
3. Keep Account Resources limited to this account. Never add Billing or
   API Tokens. Continue to summary > Update token. The token's value
   doesn't change, so the GitHub secret stays as it is.

## Where secrets go

None: editing permissions keeps the same value in `CLOUDFLARE_API_TOKEN`.

## Verify

Re-run the latest "Deploy tofu_state" workflow run; it deploys the Worker
and creates its D1 database. The `cloudflare-api-token-scopes` settings
check covers the scopes every 90 days.

## Undo or rotate

Edit the token again to drop a permission. To replace the token, see
[Rotate or revoke a secret](rotate-a-secret.md).
