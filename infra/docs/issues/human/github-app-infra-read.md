---
added: 2026-11-27
unblocks: "The github IaC stack: code security and the CI result required check on main (decision 0007, phase 2)"
runbook: standards/docs/runbooks/github-app-infra-read.md
---

## Requirement

Create a second, read-only GitHub App (Administration: read, Metadata: read) installed only on `jimhillr/monorepo`, and store its key as the repository secret `GH_INFRA_READ_APP_KEY` and its ID as the repository variable `GH_INFRA_READ_APP_ID`. Then approve the "apply credentials (github)" run, which turns on secret scanning with push protection, Dependabot alerts and security updates, and adds the `CI` ruleset requiring `CI result` on `main`.

## Why it can't be automated

GitHub has no API that creates an app or its private key, and the infra GitHub App's key can't be used for plans: it can write secrets, so it lives only in `infra-credentials`, where every run waits for Jim.

## Runbook

[Create the read-only infra GitHub App](../../../../standards/docs/runbooks/github-app-infra-read.md)
