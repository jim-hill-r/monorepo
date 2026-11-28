# backups requirements

What the backups promise Jim and every project that holds data. The format
and the rules are in
[standards/docs/evals.md](../../standards/docs/evals.md). The targets come
from [decision 0030](../../standards/docs/decisions/0030-backups-and-disaster-recovery.md).

Drafted by an agent on 2026-11-28 with the decision record, for Jim's
approval. No evals yet: the offline copy's settings check and the monthly
restore drill (phase 2) are how they are checked.

## BACK-001: The repo, with the cookbook, survives losing GitHub

- Priority: must
- Why: The repo holds all our work and the cookbook content, which is the
  product and a future book, and GitHub is its only online home. Jim
  chose an offline copy over a second online one (decision 0030, part 2).
- Accept: Jim's answer to the `offline-backup-copy` settings check is less
  than 100 days old and says the copy's `cookbook/content/` folder has
  recipes in it.

## BACK-002: User data in D1 can be restored to any day in the last 90

- Priority: must
- Why: Users wrote it, so the repo can't rebuild it.
- Accept: every D1 database a `wrangler.toml` binds has an encrypted
  export less than 24 hours old, and the newest one decrypts with the
  backup key, loads into SQLite and passes `PRAGMA integrity_check`.

## BACK-003: Nothing readable leaves the runner

- Priority: must
- Why: The repo is public, so its workflow artifacts are too.
- Accept: every uploaded export is age-encrypted, and the drill fails if
  any artifact of the backup job parses as SQL or JSON.

## BACK-004: Every stateful resource has a recovery plan

- Priority: should
- Why: A database added without a backup is the one that gets lost.
- Accept: every D1 database and KV namespace a `wrangler.toml` or the
  `cloudflare` stack names has a row in `backups/docs/inventory.md` with a tier.
