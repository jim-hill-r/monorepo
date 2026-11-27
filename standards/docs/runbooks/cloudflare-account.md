# Set up Cloudflare

- **Kind:** one-time (done; keep for a rebuild)
- **Status:** done before 2026-11-26
- **Time:** about 15 minutes from scratch
- **Cost:** $0 on the Free plan with no payment method

## What it's for

Cloudflare hosts every site (Pages, `jimhillr-<project>.pages.dev`),
Workers such as `jimhillr-tofu-state` and `jimhillr-telemetry`, Workers
Analytics Engine (the `telemetry` dataset) and Web Analytics. Deploys in
`push-main.yml`, `nightly-cd.yml` and `tofu-state-deploy.yml` use one
deploy token.

## Why a person

Creating the account and the first token needs a signed-in person; a
token can't be made without one that can make tokens.

## Before you start

- An email address for the account.

## Steps

1. Sign up at dash.cloudflare.com on the Free plan. Add no payment method
   and no Workers Paid.
2. Copy the Account ID (Workers & Pages > Overview, right side).
3. My Profile > API Tokens > Create Token > Custom token, named for
   example `github-deploy`, account-scoped to this account only, with:
   Cloudflare Pages: Edit, Workers Scripts: Edit, D1: Edit and Account
   Analytics: Read ([Change the Cloudflare deploy token's
   permissions](cloudflare-deploy-token.md)). Never Billing or API Tokens.
   Set an expiry and note it.
4. Add both to GitHub as below.
5. Turn on Workers Analytics Engine: Workers & Pages > Analytics Engine
   (`https://dash.cloudflare.com/<account id>/workers/analytics-engine`),
   then the button that enables it. Cloudflare has no API for this switch,
   and until it is on, deploying a Worker with an Analytics Engine binding
   (`telemetry`) fails with "You need to enable Analytics Engine" (code
   10089). The Free plan allows 100,000 data points a day; add no payment
   method. (Jim turned it on 2026-11-27.)
6. Connect Cloudflare read-only in Claude ([Set up Claude](claude-setup.md)).

## Where secrets go

Repository secrets: `CLOUDFLARE_API_TOKEN` (the token) and
`CLOUDFLARE_ACCOUNT_ID` (the account ID; not strictly secret, but kept
with the token).

## Verify

A push to `main` that changes a site deploys it, and the telemetry
deploy passes (`https://jimhillr-telemetry.jim-hill-r.workers.dev/event`
answers a `GET` with 405). The settings checks
`github-secret-cloudflare-*`, `cloudflare-api-token-scopes` and
`cloudflare-billing` answer "ok".

## Undo or rotate

[Rotate or revoke a secret](rotate-a-secret.md): Roll the token in My
Profile > API Tokens and overwrite `CLOUDFLARE_API_TOKEN`.
