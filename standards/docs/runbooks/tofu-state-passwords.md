# Make the OpenTofu state passwords

- **Kind:** one-time
- **Status:** open (`issues/human/tofu-state-passwords.md`). The
  token-minting workflow (Jim, 2026-11-27) is expected to generate these.
- **Time:** about 5 minutes
- **Cost:** $0

## What it's for

The `jimhillr-tofu-state` Worker
([decision 0009](../decisions/0009-opentofu-state-http-backend.md)) holds
OpenTofu state and accepts only two passwords: a read one for PR plans and
a write one for applies on `main`.

## Why a person

Until the minting workflow exists, nothing in automation can make a
secret without its value passing through git, a log or state. A password
manager can, and the value goes straight into GitHub.

## Before you start

- Signed in to GitHub as `jimhillr`.
- The `infra` environment exists ([Create the Auth0 IaC
  applications](auth0-iac-credentials.md), step 3).

## Steps

1. Generate two random passwords of 32 or more characters with a password
   manager's generator.
2. Add them to GitHub as below, pasting each one straight in.
3. Don't save them anywhere: the Worker's deploy copies them from GitHub.

## Where secrets go

- `TOFU_STATE_READ_PASSWORD`: a repository secret.
- `TOFU_STATE_WRITE_PASSWORD`: a secret of the `infra` environment only.

## Verify

The next run of the "Deploy tofu_state" workflow passes, and an `infra`
PR plan reads state. The `github-secrets-tofu-state` settings check covers
it every 90 days.

## Undo or rotate

Overwrite the secret with a new password and re-run "Deploy tofu_state"
so the Worker picks it up.
