---
added: 2026-11-26
runbook: standards/docs/runbooks/github-code-security.md
---

## Requirement

In GitHub, Settings > Code security, turn on secret scanning with push protection, Dependabot alerts, Dependabot security updates and private vulnerability reporting (all free on this public repo).

PR #654. The server-side net for the threat check's secret scanning and dependency audit, see standards/docs/plans/2026-11-threat-check.md.

## Why it can't be automated

These repository settings need Admin, which `jimhillr-agt` doesn't have. A later `github` stack may manage them, applied only in `infra-credentials`.

## Runbook

[Turn on GitHub code security](../../standards/docs/runbooks/github-code-security.md)
