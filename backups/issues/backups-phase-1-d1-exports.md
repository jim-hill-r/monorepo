---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Backups / Phase 1: encrypted Cloudflare exports"
order: 10
added: 2026-11-28
---

Once the human task `make-the-backup-key` is done and `backups/keys/backups.age.pub` is committed, add `.github/workflows/backups-nightly.yml` ([decision 0030](../../standards/docs/decisions/0030-backups-and-disaster-recovery.md), part 3) in a new `backups` GitHub environment (added to the `github` IaC stack; a credential change, so Jim approves the `infra-credentials` run): `wrangler d1 export` every D1 database bound in a `wrangler.toml` (today `jimhillr-tofu-state`), encrypt each with age to the public key, and upload them as one Actions artifact kept 90 days. Never upload an unencrypted file; add a test that the job refuses to. Add age to the tech radar (an evaluated tool with this project as its exemplar) and the BACK-002 and BACK-003 evals, through a `chores backup` command (Rust, no new .sh). Update backups/docs/cost.toml, threat.toml and the user guide, and turn `backups` from a docs project into whatever the command needs. Write the Time Travel and export restore steps into standards/docs/runbooks/disaster-recovery.md, tried once against a scratch local D1.
