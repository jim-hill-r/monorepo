---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 100
added: 2026-11-26
---

Threat check phase 3. Add `threat summary --base <ref>` and a PR CI step that writes the security summary to the job summary and sets the `security` label, plus `needs-human` when a PR weakens security (more public, new secret or auth provider, `[auth]` change, wider workflow permissions or `pull_request_target`, a new gitleaks or advisory exception). Make `hooks.sh` run `threat-check.sh --projects <affected>` after the cost check, with cases in `standards/tests/hooks.test.sh`.
