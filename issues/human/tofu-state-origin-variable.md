---
added: 2026-11-27
unblocks: "the infra plan, apply and drift workflows (infra-plan.yml, infra-apply.yml, infra-drift.yml), which find state at this URL"
---

After the first run of the "Deploy tofu_state" workflow, copy the Worker URL from its job summary (`https://jimhillr-tofu-state.<subdomain>.workers.dev`) and add it in GitHub (jimhillr/monorepo > Settings > Secrets and variables > Actions > Variables) as the repository variable `TOFU_STATE_ORIGIN`. It is not a secret, which is why it is a variable. Cost: $0, about 2 minutes.
