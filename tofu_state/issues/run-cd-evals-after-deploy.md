---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Evals roadmap / Phase 2: Cover every project"
order: 100
added: 2026-11-27
---

Run tofu_state's `cd` evals after each deploy. tofu_state deploys from `tofu-state-deploy.yml`, not `push-main.yml`, and has no `DEPLOY_URL`, so the `http` evals in `evals.toml` never run and TOFU-001 and TOFU-002 have no production results. After `scripts/smoke.sh`, run `EVALS_STAGE=cd bash standards/scripts/evals.sh run` and store the results on the `evals-results` branch the way `evals-store.yml` does. Then give TOFU-003 (locking) a production eval: a check with the read password that `LOCK` answers 403, as `smoke.sh` does, either as a new eval kind that can send the password from a secret or by having `smoke.sh` write a report the engine reads.
