---
kind: fix
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 95
added: 2026-11-27
---

Clear the SEC-005 (zizmor) findings left after phase 3, all in workflows
that hold credentials, so they get their own `credential-change` PR
(decision 0010) rather than riding on the zizmor rollout:

- `github-app`: the five `actions/create-github-app-token` steps in
  `infra-apply.yml`, `infra-drift.yml`, `infra-mint.yml` and
  `infra-plan.yml` mint a token with every permission the installation has.
  Add `permission-<name>` inputs for exactly what each step needs. This
  narrows token scopes, so it needs Jim's second approval.
- `template-injection`: `${{ matrix.stack }}` inside `run:` in the same infra
  workflows (12 places). The matrix comes from `ls infra/out`, so it isn't
  attacker-controlled, but move it to `env: STACK: ${{ matrix.stack }}` and
  use `"$STACK"` so the warning goes away.
- `excessive-permissions`: `all-codeql.yml` grants `security-events: write`
  to the whole workflow; move it to the two jobs.

Run `bash standards/scripts/workflows-check.sh --all` to see them.
