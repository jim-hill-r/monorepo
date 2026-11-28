---
added: 2026-11-28
unblocks: encrypted nightly D1 exports (backups phase 2)
runbook: standards/docs/runbooks/backup-encryption-key.md
---

## Requirement

Make an age key pair, keep the private key in your password manager and on paper, store it as the `backups` environment secret `BACKUP_AGE_KEY` for the monthly drill, and give an agent the public key to commit as `backups/keys/backups.age.pub`. The repo is public, so every nightly D1 export is encrypted to this key before upload, and only you can decrypt it if GitHub is gone.

## Why it can't be automated

The private key must exist outside GitHub and Cloudflare to be any use in a disaster, so a person has to see and keep it. Infra Mint pipes every secret it makes straight into GitHub, where no one sees it, which is exactly what this key must not be limited to.

## Runbook

[Make the backup encryption key](../../standards/docs/runbooks/backup-encryption-key.md)
