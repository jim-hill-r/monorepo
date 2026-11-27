# Rotate or revoke a secret

- **Kind:** on demand (a token nears expiry, a check says it's wrong, or a
  secret may have leaked)
- **Status:** ongoing
- **Time:** about 5 minutes per secret
- **Cost:** $0

## What it's for

Every secret is made by a person (or the minting workflow) and stored only
as a GitHub or vendor secret, never in git or OpenTofu state (AGENTS.md).
Rotating keeps a leaked or expiring one from breaking or exposing anything.

## Why a person

Agents hold no token that can make or change credentials. The minting
workflow, once built, rotates the ones it made in a run you approve
([Approve a credential change run](approve-credential-run.md)).

The schedule for each secret, and which ones Infra Mint rotates for you
every quarter, is in [the rotation plan](../secret-rotation.md).

## Before you start

- Find the secret's runbook in [README.md](README.md): it says which
  vendor page makes it and where it's stored.

## Steps

1. If it may have leaked, revoke it first at the vendor (Cloudflare: My
   Profile > API Tokens > Roll or Delete; Auth0: the app > Rotate secret;
   GitHub App: Private keys > Delete; PostHog: Personal API keys > Roll).
2. Make the new value on the same page.
3. Overwrite the GitHub secret of the same name, in the same place
   (repository, `infra` or `infra-credentials`).
4. Re-run the workflow that uses it.
5. If a value ever reached git, the gitleaks scan (SEC-001) will flag it;
   revoking it at the vendor is what matters, since history can't be
   relied on to forget it.

## Where secrets go

The same name and place as before.

## Verify

The workflow that uses it passes, and the old value is gone from the
vendor's list.

## Undo or rotate

None: an old value that's been revoked can't be brought back. Make another.
