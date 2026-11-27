# Turn on GitHub code security

- **Kind:** one-time
- **Status:** open (`issues/human/github-settings-code-security-turn-secret.md`)
- **Time:** about 3 minutes
- **Cost:** $0. All of these are free on a public repo.

## What it's for

The server-side net behind the repo's own secret scan and dependency audit
([threat check plan](../plans/2026-11-threat-check.md)): GitHub blocks a
push that contains a known secret, raises Dependabot alerts and opens
security update PRs, and lets outsiders report a vulnerability privately.

## Why a person

These are repository settings that need Admin, which `jimhillr-agt`
doesn't have. A later `github` stack in `infra` may manage them, applied
only in `infra-credentials`.

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
