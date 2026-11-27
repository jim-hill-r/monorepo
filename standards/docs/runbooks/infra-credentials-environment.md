# Set up the infra environments and bootstrap credentials

- **Kind:** one-time (again only to rotate a bootstrap credential)
- **Status:** open (`issues/human/infra-credentials-environment.md`)
- **Time:** about 30 minutes
- **Cost:** $0. Every item is on a free plan, and each Infra Mint run uses one Auth0 Machine to Machine token out of the free 1,000 a month.

## What it's for

The `infra` and `infra-credentials` environments, and the three bootstrap credentials that the Infra Mint workflow makes every other infra secret from (decision 0011). Until this is done, every infra workflow skips its stacks, and the state Worker can't deploy.

## Why a person

Everything else is minted from these three credentials, so they come first. Creating each one needs a signed-in person, and they must belong to Jim, behind his approval on `infra-credentials`, so an agent can never approve or widen its own access (decision 0010).

## Before you start

- Accounts:
  - GitHub as `jimhillr`, signed in with a passkey or security key.
  - The Auth0 tenant admin.
  - The Cloudflare account Super Administrator.
- Permissions: admin of `jimhillr/monorepo`.
- Time: about 30 minutes.
- Cost: $0. Everything is on a free plan, and each mint run uses one Auth0 Machine to Machine token out of the free 1,000 a month.

## Steps

1. **Create the `infra` environment.** In jimhillr/monorepo > Settings > Environments > New environment, name it `infra`. Set Deployment branches to Selected branches, add `main`, and add no reviewers.

2. **Create the `infra-credentials` environment.** Use the same page, with these settings:
   - Required reviewers: `jimhillr` only.
   - "Allow administrators to bypass configured protection rules": off.
   - Deployment branches: `main` only.
   - No wait timer.

3. **Set up the GitHub App.** Follow [github-app-infra.md](github-app-infra.md).

4. **Create the Auth0 bootstrap app.** In manage.auth0.com > Applications > Create Application:
   - Choose Machine to Machine and name it `infra-credentials`.
   - Authorize it for the Auth0 Management API with exactly these scopes: `read:clients`, `update:clients`, `create:clients`, `update:client_keys`, `read:client_grants`, `create:client_grants` and `update:client_grants`.
   - Store its domain, client ID and secret as shown under "Where secrets go".

5. **Create the Cloudflare bootstrap token.** In dash.cloudflare.com > Manage Account > Account API Tokens > Create Token > Custom token:
   - Name it `jimhillr-infra-credentials`.
   - Give it the one permission Account > Account API Tokens > Edit, for this account only.
   - Set its TTL end date about 200 days out, so a missed rotation makes it expire ([rotation plan](../secret-rotation.md)).
   - Store it as shown under "Where secrets go".

6. **Run the first mint.** In Actions > Infra Mint > Run workflow, on `main`, set "what" to `all` and "rotate" to `CLOUDFLARE_API_TOKEN`. That replaces your hand-made deploy token with the minted `jimhillr-deploy`. Approve the run when GitHub asks. Its summary lists every secret it set, and it queues Deploy tofu_state, which needs no approval.

7. **Delete the old token.** Once Deploy tofu_state and the next Pages deploy are green, delete your old hand-made token in My Profile > API Tokens. Automation never deletes, so this one is yours.

8. **Set the Auth0 budget.** Reply in the Claude project with a number for `auth0_m2m_tokens_per_month` in `standards/radar/limits.toml`. It is 0 today; about 60 covers plans, applies, drift and mint runs. An agent makes the edit, since only you set budgets.

## Where secrets go

All of these are secrets of the `infra-credentials` environment:

| Value | Secret |
| --- | --- |
| Auth0 tenant domain (`<tenant>.<region>.auth0.com`) | `AUTH0_INFRA_CREDENTIALS_DOMAIN` |
| Auth0 `infra-credentials` client ID | `AUTH0_INFRA_CREDENTIALS_CLIENT_ID` |
| Auth0 `infra-credentials` client secret | `AUTH0_INFRA_CREDENTIALS_CLIENT_SECRET` |
| Cloudflare `jimhillr-infra-credentials` token | `CLOUDFLARE_INFRA_CREDENTIALS_TOKEN` |
| GitHub App key and ID | see [github-app-infra.md](github-app-infra.md) |

None of these may be a repository secret, and none should be stored anywhere else. Copy each value from the vendor page straight into GitHub and keep no copy.

## Verify

- Settings > Environments shows:
  - `infra`, limited to `main`.
  - `infra-credentials`, with `jimhillr` as reviewer, admin bypass off, and limited to `main`.
- The Infra Mint run is green, and its summary lists the state passwords, the Cloudflare and Auth0 tokens, `TOFU_STATE_ORIGIN` and the Android key.
- Deploy tofu_state is green, and its smoke check passes.
- The next Infra Apply run on `main` plans each stack instead of skipping it.

Reply in the Claude project when you're done, and an agent closes the issue. These settings checks in `standards/settings.toml` then come round every 90 days:
- `github-infra-credentials-environment`
- `cloudflare-infra-credentials-token`
- `auth0-infra-apps`
- `github-app-infra`

## Undo or rotate

- **Rotate a minted secret:** run Infra Mint with its name in "rotate". The state passphrase is the exception, because it encrypts state.
- **Rotate a bootstrap credential:** make a new one on the vendor page, replace its secret in `infra-credentials`, then delete the old one on the vendor page.
- **After a leak:** revoke the bootstrap credential on the vendor page first. Then run Infra Mint with every minted name in "rotate".
- **Undo everything:** delete the two environments, the Auth0 apps `infra-credentials`, `infra-read` and `infra-apply`, and the `jimhillr-*` Cloudflare account tokens.
