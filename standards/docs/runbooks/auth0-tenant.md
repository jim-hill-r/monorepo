# Set up Auth0

- **Kind:** one-time (done; keep for a rebuild)
- **Status:** done before 2026-11-26. Its URLs move to the `auth0` stack in
  `infra` once [Create the Auth0 IaC
  applications](auth0-iac-credentials.md) is done.
- **Time:** about 15 minutes from scratch
- **Cost:** $0 on the Free plan (25,000 monthly active users, 1,000 M2M
  tokens a month). Never add a custom domain: it asks for a card.

## What it's for

Sign-in for Cahokia and Cookbook through `auth_sdk`. Auth0 is an approved
exception vendor.

## Why a person

Creating the tenant needs a signed-in person, and until the IaC apps
exist nothing can create applications.

## Before you start

- An email address for the account.

## Steps

1. Sign up at auth0.com on the Free plan. The tenant is
   `dev-jdadpn4pckxevrv5.us.auth0.com`; it's in `AUTH0_DOMAIN` in
   `cahokia/web/src/main.rs` and `cookbook/web/src/main.rs`.
2. Applications > Create Application > Single Page Application, named
   `cahokia`. Allowed Callback URLs: `https://jimhillr-cahokia.pages.dev/`,
   `https://*.jimhillr-cahokia.pages.dev/`, `http://localhost:8080/`.
   Allowed Web Origins and Allowed Logout URLs: the same without the
   trailing slash. Its client ID goes in `CLIENT_ID` in
   `cahokia/web/src/main.rs` (it's public).
3. The same for `cookbook`, with `jimhillr-cookbook` in place of
   `jimhillr-cahokia`, and `CLIENT_ID` in `cookbook/web/src/main.rs`.
4. Settings > Subscription: Free, no card.

The URL lists here mirror `infra/out/auth0/main.tf.json`; once the stack
applies, that file is the source and the dashboard edits stop.

## Where secrets go

None: SPA client IDs are public. The Management API apps are in [Create
the Auth0 IaC applications](auth0-iac-credentials.md) and [Set up the
infra-credentials environment](infra-credentials-environment.md).

## Verify

Sign in works on `jimhillr-cahokia.pages.dev` and
`jimhillr-cookbook.pages.dev`. The settings checks `auth0-cahokia-app` and
`auth0-billing` answer "ok".

## Undo or rotate

Nothing to rotate for an SPA. Deleting an application breaks that site's
sign-in.
