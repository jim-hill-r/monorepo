# Runbook: create the infra GitHub App

## When

Once, before the first run of the infra minting workflow, and again only to rotate or revoke the key. The issue is [issues/human/github-app-infra-credentials.md](../../../issues/human/github-app-infra-credentials.md).

## Why a person

GitHub has no API for creating an app or its private key without an existing credential that has these rights. This app is the first credential the automation stands on, and it must belong to Jim so that only he can widen it (decision 0010).

## Before you start

- Accounts: GitHub, signed in as `jimhillr` with a passkey or security key.
- Permissions: admin of `jimhillr/monorepo`. The `infra-credentials` environment must exist (`issues/human/infra-credentials-environment.md`).
- About 10 minutes.
- Cost: $0, because GitHub Apps are free.

## Steps

1. Go to github.com > your profile > Settings > Developer settings > GitHub Apps > New GitHub App. Name it something like `jimhillr-infra`. Owner: `jimhillr`. Turn off the webhook. Under "Where can this GitHub App be installed?", pick **Only on this account**.
2. Under Repository permissions, give it:
   - **Secrets**: Read and write
   - **Environments**: Read and write
   - **Variables**: Read and write
   - **Administration**: Read and write (only for the later `github` stack: code security settings and the ruleset's required check, never its bypass list, IAC-005)
   - **Metadata**: Read-only (GitHub requires it)

   Give it nothing else, and no organization or account permissions.
3. Install it on **Only select repositories** > `jimhillr/monorepo`.
4. Generate a private key, which downloads a `.pem` file, and store it as in "Where secrets go". Then delete the `.pem` file. Don't keep a copy anywhere: you can make a new key from the app page at any time.

## Where secrets go

| What | Where | Kind |
| --- | --- | --- |
| The whole `.pem` private key | jimhillr/monorepo > Settings > Environments > `infra-credentials` | secret `GH_INFRA_CREDENTIALS_APP_KEY` |
| The App ID, from the app's General page | the same environment | variable `GH_INFRA_CREDENTIALS_APP_ID` |

Never a repository secret, and no copy anywhere else.

## Verify

- github.com/settings/installations lists the app with access to `jimhillr/monorepo` only.
- The app's permissions page shows exactly the five permissions above.
- `infra-credentials` has the secret and the variable, and the `.pem` file is gone from your computer.

Reply in the Claude project when you're done. An agent deletes the issue, and the
`github-app-infra` settings check in `standards/settings.toml` then comes round every 90 days.

## Undo or rotate

- Rotate: on the app's General page, generate a new private key, replace the secret, then delete the old key there.
- Revoke after a leak: delete the key on the app page first, so the app can't act until a new key is stored.
- Undo: uninstall and delete the app (Settings > Developer settings > GitHub Apps > the app > Advanced), then delete the secret and the variable.
