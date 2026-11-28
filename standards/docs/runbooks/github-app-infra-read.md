# Create the read-only infra GitHub App

- **Kind:** one-time, and again only to rotate or revoke the key
- **Status:** open (`infra/docs/issues/human/github-app-infra-read.md`)
- **Time:** about 10 minutes
- **Cost:** $0, because GitHub Apps are free

## What it's for

The `github` stack in `infra` manages this repository's settings: code
security and the `CI` ruleset that requires the `CI result` and `Human review` checks on
`main`. OpenTofu has to read those settings before it can plan, and
reading them needs Administration: read, which the workflow's own
`GITHUB_TOKEN` can never have. The infra GitHub App
([github-app-infra.md](github-app-infra.md)) can read them, but it can
also write every secret, so its key lives only in `infra-credentials`,
where every run waits for Jim (decision 0010). A PR plan can't wait for
that.

This second app can only read. Its key is a repository secret, so PR
plans, the plan on `main` and the weekly drift check can mint a token from
it without asking. Until it exists, those workflows skip the `github`
stack with a notice, and nothing in it is applied.

## Why a person

GitHub has no API that creates an app or its private key; the manifest
flow needs a person signed in to a browser. Like the infra GitHub App, it
belongs to Jim, so only he can widen it.

## Before you start

- Signed in to GitHub as `jimhillr` with a passkey or security key.
- Admin of `jimhillr/monorepo`.

## Steps

1. github.com > your profile > Settings > Developer settings > GitHub Apps >
   New GitHub App. Name it something like `jimhillr-infra-read`. Turn off
   the webhook. Under "Where can this GitHub App be installed?", pick
   **Only on this account**.
2. Under Repository permissions, give it:
   - **Administration**: Read-only
   - **Metadata**: Read-only (GitHub requires it)

   Nothing else: no write permission of any kind, and no organization or
   account permissions.
3. Install it on **Only select repositories** > `jimhillr/monorepo`.
4. Generate a private key, which downloads a `.pem` file, and store it as
   in "Where secrets go". Then delete the `.pem` file.
5. Approve the next **Infra Apply** run's "apply credentials (github)" job
   after reading its plan (see [approve-credential-run.md](approve-credential-run.md)).
   A merge to `main` that changes `infra/` starts one, or run Infra Apply
   by hand from the Actions tab.

## Where secrets go

| What | Where | Kind |
| --- | --- | --- |
| The whole `.pem` private key | jimhillr/monorepo > Settings > Secrets and variables > Actions | repository secret `GH_INFRA_READ_APP_KEY` |
| The App ID, from the app's General page | the same page, Variables tab | repository variable `GH_INFRA_READ_APP_ID` |

A repository secret is fine here because the app can only read settings,
never secrets. Never give this app a write permission: every PR's code can
reach its key.

## Verify

- github.com/settings/installations lists the app with access to
  `jimhillr/monorepo` only, and its permissions page shows only
  Administration: read and Metadata: read.
- The next Infra Plan run on a PR that changes `infra/` has a
  "plan (github)" job with a plan in its summary, instead of the notice
  "github is not set up yet".
- If the plan fails reading code security or Dependabot settings with a
  403, the app is missing Administration: read.

Tell the IaC thread "done" so an agent deletes the issue.

## Undo or rotate

- Rotate: on the app's General page, generate a new private key, replace
  the `GH_INFRA_READ_APP_KEY` secret, then delete the old key there.
- Revoke after a leak: delete the key on the app page first. A leaked key
  shows settings, not secrets, but rotate it anyway.
- Undo: uninstall and delete the app, then delete the secret and the
  variable. The `github` stack then skips again.
