# Set up the infra-credentials environment

- **Kind:** one-time
- **Status:** open (`issues/human/infra-credentials-environment.md`)
- **Time:** about 20 minutes
- **Cost:** $0. Environments with required reviewers are free on public
  repos (they'd need GitHub Pro, $4 a month, if the repo went private).
  Each credential change adds one Actions job of a minute or less.

## What it's for

The second approval for credential changes
([decision 0010](../decisions/0010-credential-second-approval.md)). Tokens
that can create or change credentials live only in this environment, so a
job that uses them waits until Jim approves the run. Until it exists, IaC
can't apply any credential change.

## Why a person

It is the gate on agents, so agents must not be able to make or change it:
`jimhillr-agt` has Write, not Admin, and creating environments and their
protection rules needs Admin. Its tokens are the first credentials, which
nothing can mint before they exist.

## Before you start

- Signed in to GitHub as `jimhillr` (an admin of jimhillr/monorepo).
- Signed in to the Auth0 dashboard for the tenant
  `dev-jdadpn4pckxevrv5.us.auth0.com`.
- [Create the Auth0 IaC applications](auth0-iac-credentials.md) done or
  done in the same sitting, since it creates the `infra` environment.

## Steps

1. jimhillr/monorepo > Settings > Environments > New environment, named
   `infra-credentials`.
   - Required reviewers: `jimhillr` only. Leave "Prevent self-review" off
     (you are the only reviewer).
   - Turn **off** "Allow administrators to bypass configured protection
     rules".
   - Deployment branches and tags: Selected branches, add `main` only.
   - No wait timer. Save protection rules.
2. In Auth0 (manage.auth0.com > Applications > Applications > Create
   Application), create a Machine to Machine app named `infra-credentials`,
   authorized for the Auth0 Management API with these scopes only:
   `read:clients`, `update:clients`, `create:clients`, `update:client_keys`,
   `read:client_grants`, `create:client_grants`, `update:client_grants`.
3. From the app's Settings tab, add its Domain, Client ID and Client
   Secret as secrets of `infra-credentials` (see below).
4. When the Cloudflare and GitHub stacks arrive, add their
   credential-capable tokens here too (Cloudflare API Tokens: Edit and
   Access: Apps and Policies: Edit; the infra GitHub App's key from
   `github-app-infra.md`). Never as repository secrets.
5. github.com > Settings > Password and authentication: make sure
   `jimhillr` signs in with a passkey or security key, since this account's
   sign-in is now what protects every credential.

## Where secrets go

All in jimhillr/monorepo > Settings > Environments > `infra-credentials` >
Environment secrets, nowhere else:

- `AUTH0_INFRA_CREDENTIALS_DOMAIN`
- `AUTH0_INFRA_CREDENTIALS_CLIENT_ID`
- `AUTH0_INFRA_CREDENTIALS_CLIENT_SECRET`

Copy the client secret straight from Auth0 into GitHub; don't save it
anywhere else.

## Verify

- The environment page shows "Required reviewers: jimhillr", main only, and
  no admin bypass.
- Repository secrets (Settings > Secrets and variables > Actions) list no
  `AUTH0_INFRA_CREDENTIALS_*`.
- Tell any thread "done"; it deletes the issue. The settings checks
  `github-infra-credentials-environment`, `github-agent-role` and
  `github-claude-app-permissions` then come round every 90 days.

## Undo or rotate

To rotate the Auth0 app's secret: Auth0 > the app > Settings > Rotate
secret, then overwrite `AUTH0_INFRA_CREDENTIALS_CLIENT_SECRET`. Deleting
the environment removes the gate and its secrets, so only do that if IaC is
being retired.
