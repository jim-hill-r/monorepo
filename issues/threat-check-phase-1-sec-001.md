---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 20
added: 2026-11-26
---

Threat check phase 1. Add SEC-001: gitleaks pinned in the root `mise.toml`, run on the staged diff in pre-commit and on the pushed range in pre-push (a warning when it can't be installed, per git-hooks.md), on the PR range in PR CI as an error, and over the full history in the weekly `all-free-tier-usage.yml` run. Add our rules for `RAPIDAPI_`, PostHog `phx_` keys and Auth0 management tokens, and a `.gitleaksignore` convention. Run a one-off full-history scan first; for any real secret, add a human task in `issues/human/` to rotate it instead of rewriting history.
