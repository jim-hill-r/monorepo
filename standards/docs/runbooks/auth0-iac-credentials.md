# Create the Auth0 IaC applications

- **Kind:** one-time
- **Status:** Replaced by the Infra Mint workflow on 2026-11-27 ([decision 0011](../decisions/0011-mint-secrets-by-workflow.md)), which makes this for you. Kept only to show what a hand-made version looked like; for a rebuild, follow [infra-credentials-environment.md](infra-credentials-environment.md).
- **Time:** about 15 minutes
- **Cost:** $0. Each plan and apply gets one Management API token: about 30
  to 60 a month of the free 1,000 M2M tokens.

## What it's for

IaC phase 1 ([decision 0007](../decisions/0007-infrastructure-as-code.md))
reads and updates the Auth0 applications, so callback, origin and logout
URLs come from code instead of the dashboard. PR plans use a read-only app;
applies on `main` use one that can update URLs but not credentials.

## Why a person

These are the first Auth0 credentials automation gets, so nothing can make
them before they exist, and an Auth0 client secret must never land in
OpenTofu state (IAC-002). Creating the `infra` environment needs repo
Admin, which agents don't have.

## Before you start

- Signed in to manage.auth0.com (tenant `dev-jdadpn4pckxevrv5.us.auth0.com`)
  and to GitHub as `jimhillr`.

## Steps

1. Auth0 > Applications > Applications > Create Application > Machine to
   Machine, named `infra-read`. Authorize it for the Auth0 Management API
   with only `read:clients`.
2. Create a second one, `infra-apply`, with only `read:clients` and
   `update:clients`.
3. GitHub: jimhillr/monorepo > Settings > Environments > New environment
   `infra`. Deployment branches: `main` only. No reviewers (everyday IaC
   applies on merge).
4. Add the secrets below, copying each value from the app's Settings tab.
5. Add `TOFU_STATE_PASSPHRASE` to `infra`: a long random string (32+
   characters from a password manager's generator).
6. Reply with a budget for `auth0_m2m_tokens_per_month` (about 60 is
   enough); an agent sets it in `standards/radar/limits.toml`. See
   [Set or raise a budget](set-a-budget.md).

## Where secrets go

- Repository secrets: `AUTH0_INFRA_READ_DOMAIN`, `AUTH0_INFRA_READ_CLIENT_ID`,
  `AUTH0_INFRA_READ_CLIENT_SECRET`.
- `infra` environment secrets: `AUTH0_INFRA_APPLY_DOMAIN`,
  `AUTH0_INFRA_APPLY_CLIENT_ID`, `AUTH0_INFRA_APPLY_CLIENT_SECRET`,
  `TOFU_STATE_PASSPHRASE`.

Don't keep a copy anywhere else.

## Verify

The next PR that touches `infra` shows an Auth0 plan in its job summary
instead of a missing-credentials error. The read app is also what the
resource cleanup plan's phase 4 uses.

## Undo or rotate

Auth0 > the app > Settings > Rotate secret, then overwrite the matching
`*_CLIENT_SECRET`. To retire an app, delete it in Auth0 and its secrets in
GitHub. `TOFU_STATE_PASSPHRASE` encrypts state: changing it needs state
re-encrypted first, so ask an agent before rotating it.
