---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Backups / Phase 2: encrypted Cloudflare exports"
order: 20
added: 2026-11-28
---

Once the human task `make-the-backup-key` is done and `backups/keys/backups.age.pub` is committed, extend the nightly job ([decision 0030](../../standards/docs/decisions/0030-backups-and-disaster-recovery.md), part 3): `wrangler d1 export` every D1 database bound in a `wrangler.toml` (today `jimhillr-tofu-state`), encrypt each with age to the public key, and upload them as one Actions artifact kept 90 days. Never upload an unencrypted file; add a test that the job refuses to. Add age to the tech radar (an evaluated tool with this project as its exemplar) and the BACK-003 and BACK-004 evals. Write the Time Travel and export restore steps into standards/docs/runbooks/disaster-recovery.md, tried once against a scratch local D1.
