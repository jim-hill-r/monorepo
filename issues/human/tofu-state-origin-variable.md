---
added: 2026-11-27
unblocks: "the infra plan, apply and drift workflows (infra-plan.yml, infra-apply.yml, infra-drift.yml), which find state at this URL"
runbook: standards/docs/runbooks/tofu-state-origin-variable.md
---

## Requirement

After the first run of the Deploy tofu_state workflow, add the state Worker's URL (`https://jimhillr-tofu-state.<subdomain>.workers.dev`) as the repository variable `TOFU_STATE_ORIGIN`. The infra workflows find state there.

## Why it can't be automated

No workflow can write repository variables yet: the GITHUB_TOKEN can't, and the infra GitHub App that can isn't set up. The Infra Mint workflow (PR #729) will set this variable itself, and that PR deletes this task.

## Runbook

[standards/docs/runbooks/tofu-state-origin-variable.md](../../standards/docs/runbooks/tofu-state-origin-variable.md), about 2 minutes, $0.
