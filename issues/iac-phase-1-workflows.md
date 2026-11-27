---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Infrastructure as code / Phase 1: crate, Auth0 and pipeline"
order: 40
added: 2026-11-26
---

Decision 0007, items 5, 7 and 10: workflows for the `infra` project. PR CI runs `tofu plan` with the read-only Auth0 credentials and the guards, and writes the plan to the job summary. On push to `main`, a job in a protected `infra` environment (main only) plans, saves the plan, runs the guards, and a second job applies exactly that plan file. State lives in the `jimhillr-tofu-state` Worker (decision 0009, the `tofu_state` project, deployed by `tofu-state-deploy.yml`): `tofu init $(infra backend-config <stack> "$(bash tofu_state/scripts/origin.sh)")` with `TF_HTTP_PASSWORD` set to `TOFU_STATE_READ_PASSWORD` for PR plans (which run with `-lock=false`) and `TOFU_STATE_WRITE_PASSWORD` in `infra`, encrypted with the PBKDF2 key from `TOFU_STATE_PASSPHRASE` as `TF_VAR_tofu_state_passphrase`; nothing is committed to git; every infra job shares one `concurrency` group. Add a weekly `tofu plan -refresh-only -detailed-exitcode` drift job to `all-free-tier-usage.yml` that goes red on drift. Test the workflows in `monorepo/workflow_tests`. Needs the state Worker deployed and the human issues for the Auth0 credentials and the state passwords first.

If decision 0010 (a second approval for credential changes) is accepted, split the apply step in two. The plan job uses read-only tokens. It uploads the saved plan as an artifact, writes its SHA-256 to the job summary, and lists any credential changes the plan contains (from IAC-005). A stack with no credential change applies in the `infra` environment as described above. A stack with a credential change applies in a second job that uses the `infra-credentials` environment (main only, `jimhillr` as the only required reviewer). That job checks the hash and applies exactly that plan file, without re-planning. Credential-capable tokens are secrets of `infra-credentials` only, never repo secrets. The PR plan comment starts with a "Credential change" line and adds the `credential-change` label. IAC-005 is built: `infra guard <plan.json> --report report.json` writes `{"credential_change": bool, "changes": [...]}`, which picks the job.
