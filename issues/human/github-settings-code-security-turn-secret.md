---
added: 2026-11-26
runbook: standards/docs/runbooks/github-code-security.md
---

## Requirement

In GitHub, Settings > Code security, turn on private vulnerability reporting (free on this public repo). The `github` stack in `infra` turns on the rest (secret scanning with push protection, Dependabot alerts and security updates) once `github-app-infra-read.md` is done.

PR #654. The server-side net for the threat check's secret scanning and dependency audit, see standards/docs/plans/2026-11-threat-check.md.

## Why it can't be automated

It needs Admin, which `jimhillr-agt` doesn't have, and the GitHub provider (6.13.0) has no resource for private vulnerability reporting, so IaC can't do it either.

## Runbook

[Turn on GitHub code security](../../standards/docs/runbooks/github-code-security.md)
