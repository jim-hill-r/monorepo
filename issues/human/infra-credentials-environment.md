---
added: 2026-11-26
unblocks: "the second approval for credential changes (decision 0010): until this is done, IaC can't apply any credential change"
---

Set up the `infra-credentials` GitHub environment (about 20 minutes, decision 0010):

1. In jimhillr/monorepo > Settings > Environments, create `infra-credentials`. Required reviewers: `jimhillr` only. Turn "Allow administrators to bypass configured protection rules" off. Deployment branches: `main` only. No wait timer.
2. In Auth0, create a third Machine to Machine app for the Management API, "infra-credentials", with `read:clients`, `update:clients`, `create:clients`, `update:client_keys`, and `read:client_grants`, `create:client_grants`, `update:client_grants`. Add its domain, client ID and secret as `AUTH0_INFRA_CREDENTIALS_*` secrets of `infra-credentials` only. The "infra-apply" app from `auth0-iac-credentials.md` keeps only `read:clients` and `update:clients`.
3. When the Cloudflare and GitHub stacks arrive, make their credential-capable tokens (Cloudflare API Tokens: Edit and Access: Apps and Policies: Edit; a fine-grained GitHub token on `jimhillr` for this repo with Administration, Environments, Secrets and Variables) and store them only in `infra-credentials`, never as repository secrets.
4. Make sure `jimhillr` signs in with a passkey or security key.

Reply when done and an agent closes this; the settings checks `github-infra-credentials-environment`, `github-agent-role` and `github-claude-app-permissions` in `standards/settings.toml` then come round every 90 days.
