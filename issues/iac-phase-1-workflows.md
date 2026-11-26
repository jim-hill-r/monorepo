---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Infrastructure as code / Phase 1: crate, Auth0 and pipeline"
order: 40
added: 2026-11-26
---

Decision 0007, items 5, 7 and 10: workflows for the `infra` project. PR CI runs `tofu plan` with the read-only Auth0 credentials and the guards, and writes the plan to the job summary. On push to `main`, a job in a protected `infra` environment (main only) plans, saves the plan, runs the guards, and a second job applies exactly that plan file. State lives in the R2 bucket `jimhillr-infra-state` (decision 0009; the synthesized backend already points there, with `use_lockfile`), encrypted with the PBKDF2 key from `TOFU_STATE_PASSPHRASE` as `TF_VAR_tofu_state_passphrase`, and the R2 keys and endpoint as `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY` and `AWS_ENDPOINT_URL_S3` (read set for PR plans, write set in `infra`); nothing is committed to git; every infra job shares one `concurrency` group. Add a weekly `tofu plan -refresh-only -detailed-exitcode` drift job to `all-free-tier-usage.yml` that goes red on drift. Test the workflows in `monorepo/workflow_tests`. Needs the human issues for the Auth0 credentials and the R2 state bucket first.
