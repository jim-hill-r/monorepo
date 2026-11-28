# Recover from a disaster

- **Kind:** on demand (when something in [the inventory](../../../backups/docs/inventory.md) is lost), and a yearly tabletop drill
- **Status:** open (kept for every disaster; backups phases 1 to 3 fill in the restore commands)
- **Time:** about 1 hour for D1 data; up to 3 days for losing GitHub
- **Cost:** $0

## What it's for

The steps to get back what [decision 0030](../decisions/0030-backups-and-disaster-recovery.md)
backs up, in its recovery tiers. Agents do every step they can; the steps
marked **Jim** need you. Find the case that matches and follow it.

## Why a person

Rebuilding an account, a repository or the first credentials needs a
signed-in owner, and every restore that overwrites live data is a
deletion that a person approves.

## Before you start

- Your password manager with the recovery codes
  ([account-recovery-codes.md](account-recovery-codes.md)) and the backup
  key ([backup-encryption-key.md](backup-encryption-key.md)).
- Write the time it started and what is lost in an issue file (or, if
  GitHub is gone, a note) so the retro has a timeline.

## Steps

### A. A file, commit or branch is lost

1. An agent finds it in git history (`git log --all -- <path>`,
   `git reflog` on a clone) and opens a PR restoring it.
2. If history was rewritten, look for the commit in any other clone,
   then in your offline copy (a `--mirror` copy has the history), and
   push it to a new branch. **Jim** approves the PR.

### B. D1 data is wrong or deleted

1. Stop the writes: an agent turns off the product's write path (a flag,
   or a deploy of the last good version).
2. Within 7 days: the agent finds the bookmark from before the damage
   with `wrangler d1 time-travel info <db> --timestamp <time>`. **Jim**
   approves `wrangler d1 time-travel restore <db> --bookmark <id>`,
   which overwrites the database.
3. Older than 7 days, or the database is gone: the agent downloads the
   newest good export artifact from "Backups Nightly", decrypts it with
   `age -d -i <key>` and runs `wrangler d1 execute <db> --file <dump.sql>`
   into a new database, then switches the binding in a PR. **Jim**
   approves.
4. For `jimhillr-tofu-state`, try first the Worker's own 10 versions
   (tofu_state/README.md, "Rolling back a bad apply").

### C. A site or Worker is broken or deleted

1. Run nightly CD by hand (`nightly-cd.yml`, workflow dispatch) or the
   project's deploy workflow. For config: `infra-apply`.

### D. A secret is lost or leaked

Follow [rotate-a-secret.md](rotate-a-secret.md). Everything Infra Mint
makes comes back with one run; the bootstrap credentials follow
[infra-credentials-environment.md](infra-credentials-environment.md).

### E. A vendor account is locked

1. **Jim** uses that account's recovery codes or second passkey, then
   the vendor's own account recovery.
2. Once back in, rotate every credential the account holds
   (case D), because a lockout may be a compromise.

### F. GitHub is gone (account, repository, or both)

1. **Jim**: try GitHub account recovery and GitHub Support first. A
   deleted repository can often be restored by the owner within 90 days
   (Settings > Repositories > Deleted repositories).
2. If it can't come back: **Jim** creates a new account or organization
   and an empty public repository (see [github-repo-setup.md](github-repo-setup.md)).
3. From your offline copy: for a `--mirror` copy, `git push --mirror`
   to the new repository; for a ZIP, unzip it, `git init`, commit
   everything and push. Work since the copy's date is lost unless some
   other clone has it.
4. **Jim**: install the Claude GitHub App on it and point the Claude
   project at it ([claude-setup.md](claude-setup.md)).
5. An agent then: updates the repository name in the `github` stack and
   anywhere the old name is hard-coded, runs `infra-apply` for rulesets
   and environments, and asks **Jim** to set the bootstrap credentials and
   approve the Infra Mint run, which remakes every other secret. Nightly
   CD then redeploys every site.
6. If Cloudflare is gone too, follow case G as well.

### G. Cloudflare is gone

1. **Jim**: Cloudflare account recovery first.
2. Otherwise: a new account ([cloudflare-account.md](cloudflare-account.md)),
   then case D for its credentials, `infra-apply` for its config, nightly
   CD for the sites, and case B step 3 for each D1 database from the
   GitHub artifacts. Analytics and logs are not restored (tier 4).

### Yearly tabletop drill

Once a year, when the settings check asks, **Jim** and an agent read case
F step by step against today's setup, without changing anything, and the
agent fixes every step here that is wrong or missing in a PR.

## Where secrets go

Restores make no new secret except in cases D to G, which follow their
own runbooks. The backup key is used on your own computer or as
`BACKUP_AGE_KEY`, never pasted into a chat.

## Verify

The restored thing passes its project's evals (`mise run evals`), and for
the repo, the monthly drill passes against the new home. Write a short
retro in `backups/docs/retros/` with the timeline and what to change.

## Undo or rotate

A D1 restore (case B) prints the bookmark from just before it, so it can
be undone with another Time Travel restore. Nothing else here deletes.
