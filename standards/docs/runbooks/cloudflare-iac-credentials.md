# Create the Cloudflare IaC tokens

- **Kind:** one-time
- **Status:** open (`issues/human/cloudflare-iac-credentials.md`). The
  token-minting workflow (Jim, 2026-11-27) is expected to replace this.
- **Time:** about 10 minutes
- **Cost:** $0. Web Analytics is free.

## What it's for

The `cloudflare` stack in `infra` turns on Web Analytics for every
production Pages project. PR plans use `infra-read`; applies on `main` use
`infra-apply`.

## Why a person

They are the first Cloudflare tokens IaC gets. A token made by IaC would
put its value in OpenTofu state, which the rule "never store secrets in tf
state" forbids, so they're made in the dashboard (or later by the minting
workflow).

## Before you start

- Signed in to dash.cloudflare.com with access to the account.
- The `infra` environment exists ([Create the Auth0 IaC
  applications](auth0-iac-credentials.md), step 3).

## Steps

1. dash.cloudflare.com > My Profile > API Tokens > Create Token > Create
   Custom Token.
2. Name `infra-read`. Permissions: Account > Account Web Analytics > Read.
   Account Resources: Include > this account only. Set an expiry (a year
   is fine) and note it. Create, and copy the token.
3. Repeat for `infra-apply` with Account Web Analytics > Edit.
4. Give neither any API Tokens permission, so IaC can never mint or change
   tokens.
5. Add them to GitHub as below.

## Where secrets go

- `CLOUDFLARE_INFRA_READ_TOKEN`: a repository secret.
- `CLOUDFLARE_INFRA_APPLY_TOKEN`: a secret of the `infra` environment only.

## Verify

A PR touching `infra` shows a Cloudflare plan in its job summary. The
`cloudflare-infra-token-scopes` settings check covers it every 90 days.

## Undo or rotate

Before a token expires, "Roll" it in My Profile > API Tokens and overwrite
the secret ([Rotate or revoke a secret](rotate-a-secret.md)). When phase 3
adds Pages and Access, edit both tokens to add those permissions.
