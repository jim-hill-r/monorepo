# Turn on GitHub code security

- **Kind:** one-time
- **Status:** open (`issues/human/github-settings-code-security-turn-secret.md`),
  for private vulnerability reporting only. Since 2026-11-27 the `github`
  stack in `infra` manages secret scanning, push protection, Dependabot
  alerts and Dependabot security updates, once
  [the read-only infra GitHub App](github-app-infra-read.md) exists and
  Jim approves its run.
- **Time:** about 3 minutes
- **Cost:** $0. All of these are free on a public repo.

## What it's for

The server-side net behind the repo's own secret scan and dependency audit
([threat check plan](../plans/2026-11-threat-check.md)): GitHub blocks a
push that contains a known secret, raises Dependabot alerts and opens
security update PRs, and lets outsiders report a vulnerability privately.

## Why a person

These are repository settings that need Admin, which `jimhillr-agt`
doesn't have. The `github` stack in `infra` now manages all of them but
private vulnerability reporting, which the GitHub provider (6.13.0) has no
resource for. Doing the others by hand too is harmless: IaC then imports
them unchanged.

## Before you start

- Signed in to GitHub as `jimhillr`.

## Steps

1. jimhillr/monorepo > Settings > Advanced Security (called "Code security"
   on some accounts).
2. Turn on, each with its Enable button:
   - Private vulnerability reporting
   - Dependabot alerts
   - Dependabot security updates
   - Secret Protection (secret scanning), and under it Push protection
3. Leave Dependabot version updates as they are: `.github/dependabot.yml`
   drives those.

## Where secrets go

None.

## Verify

The page shows each one as enabled, and the Security tab lists Dependabot
and Secret scanning. The `github-dependabot` settings check covers the
Dependabot part every 90 days.

## Undo or rotate

Each setting has a Disable button on the same page.
