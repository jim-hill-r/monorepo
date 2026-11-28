# backups requirements

What the backups promise Jim and every project that holds data. The format
and the rules are in
[standards/docs/evals.md](../../standards/docs/evals.md). The targets come
from [decision 0030](../../standards/docs/decisions/0030-backups-and-disaster-recovery.md).

Drafted by an agent on 2026-11-28 with the decision record, for Jim's
approval. No evals yet: the monthly restore drill (phase 3) is how each is
checked.

## BACK-001: The repo, with the cookbook, survives losing GitHub

- Priority: must
- Why: The repo holds all our work and the cookbook content, which is the
  product and a future book, and today it lives only on GitHub.
- Accept: a copy of every branch less than 24 hours old is in Workers KV,
  `git bundle verify` passes on it, and a clone of it has as many files in
  `cookbook/content/` as GitHub's `main`.

## BACK-002: A copy survives losing both GitHub and Cloudflare

- Priority: should
- Why: A single compromised or closed account must not take the cookbook
  with it.
- Accept: Jim's answer to the "Offline copy" settings check is less than
  100 days old.

## BACK-003: User data in D1 can be restored to any day in the last 90

- Priority: must
- Why: Users wrote it, so the repo can't rebuild it.
- Accept: every D1 database a `wrangler.toml` binds has an encrypted
  export less than 24 hours old, and the newest one decrypts with the
  backup key, loads into SQLite and passes `PRAGMA integrity_check`.

## BACK-004: Nothing readable leaves the runner

- Priority: must
- Why: The repo is public, so its workflow artifacts are too.
- Accept: every uploaded export is age-encrypted, and the drill fails if
  any artifact of the backup job parses as SQL or JSON.

## BACK-005: Every stateful resource has a recovery plan

- Priority: should
- Why: A database added without a backup is the one that gets lost.
- Accept: every D1 database and KV namespace a `wrangler.toml` or the
  `cloudflare` stack names has a row in `backups/docs/inventory.md` with a tier.
