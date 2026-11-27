# Runbook: create the infra GitHub App

For [issues/human/github-app-infra-credentials.md](../../../issues/human/github-app-infra-credentials.md).
About 10 minutes. Cost: $0, because GitHub Apps are free.

## Before you start

The `infra-credentials` environment exists (`issues/human/infra-credentials-environment.md`),
and you are signed in as `jimhillr` with a passkey or security key.

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
4. Generate a private key, which downloads a `.pem` file. In jimhillr/monorepo > Settings > Environments > `infra-credentials` (from `infra-credentials-environment.md`), add the whole file as the secret `GH_INFRA_CREDENTIALS_APP_KEY`, and the App ID (on the app's General page) as the environment variable `GH_INFRA_CREDENTIALS_APP_ID`. Then delete the `.pem` file: don't keep a copy anywhere, since you can make a new key from the app page at any time.

## Check it worked

- github.com/settings/installations lists the app with access to `jimhillr/monorepo` only.
- The `infra-credentials` environment has the secret `GH_INFRA_CREDENTIALS_APP_KEY` and the variable `GH_INFRA_CREDENTIALS_APP_ID`.
- The `.pem` file is deleted from your computer.

Reply in the Claude project when you're done. An agent deletes the issue, and the
`github-app-infra` settings check in `standards/settings.toml` then comes round every 90 days.

## Rotating or revoking

On the app's General page, generate a new private key, replace the secret, and delete the old key.
If the key leaks, delete it there first: the app then can't act until a new key is stored.
