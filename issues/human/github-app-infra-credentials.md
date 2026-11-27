---
added: 2026-11-27
unblocks: "the infra minting workflow, which writes every other infra token and password into GitHub secrets so no one makes or sees them (Jim chose this on 2026-11-27), and later the `github` stack"
runbook: standards/docs/runbooks/github-app-infra.md
---

## Requirement

Create a private GitHub App that you own, installed only on `jimhillr/monorepo`. It needs Secrets, Environments, Variables and Administration (read and write) and Metadata (read), and its private key is stored only in the `infra-credentials` environment. The infra minting workflow uses it to write every other infra token and password into GitHub secrets, so no one makes or sees them (Jim chose this on 2026-11-27). The later `github` stack uses it to manage repository settings.

## Why it can't be automated

Creating a GitHub App and its private key needs a signed-in person: GitHub has no API for it without an existing app or token with these rights, and this app is the first credential the automation stands on. It also has to belong to you, not the agent account or the Claude GitHub App, so that only you can widen what it can do (decision 0010).

## Runbook

[standards/docs/runbooks/github-app-infra.md](../../standards/docs/runbooks/github-app-infra.md), about 10 minutes, $0.
