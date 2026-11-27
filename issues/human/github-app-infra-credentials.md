---
added: 2026-11-27
unblocks: "the infra minting workflow, which writes every other infra token and password into GitHub secrets so no one makes or sees them (Jim chose this on 2026-11-27), and later the `github` stack"
---

Create a GitHub App for infra (about 10 minutes, $0: GitHub Apps are free).

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

This must be a new app that you own, not the Claude GitHub App, so the agent account never holds these permissions (decision 0010). Each run of the minting workflow swaps the key for an installation token that expires within an hour.
