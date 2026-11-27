# Approve a credential change run

- **Kind:** on demand (each merged PR whose plan changes a credential)
- **Status:** ongoing, once `infra-credentials` exists
- **Time:** about 2 minutes
- **Cost:** $0; one Actions job of a minute or less

## What it's for

The second approval for credential changes
([decision 0010](../decisions/0010-credential-second-approval.md)): an
apply that creates or changes an Auth0 app's grants, a token's scopes, an
Access policy, or GitHub secrets, environments or rulesets runs only after
you approve it, on top of the PR approval. The same goes for each run of
the token-minting workflow.

## Why a person

It is the one check agents can't give: only `jimhillr` is a required
reviewer of `infra-credentials`, and GitHub enforces it server-side.

## Before you start

- Signed in to GitHub as `jimhillr` (web or GitHub Mobile), with your
  passkey.

## Steps

1. Open the notification "Review pending deployments", or Actions > the
   waiting run.
2. Read the plan in the run's summary: the credential resources it creates
   or changes, and the PR it came from. Check it matches what the PR said.
3. If it does: Review deployments > tick `infra-credentials` > Approve and
   deploy. If not: Reject, and say why in the PR.

## Where secrets go

None.

## Verify

The apply job turns green. A rejected run leaves everything as it was.

## Undo or rotate

Revert the PR; its revert is itself a credential change and waits for you
again.
