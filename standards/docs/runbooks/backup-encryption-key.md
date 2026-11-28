# Make the backup encryption key

- **Kind:** one-time (make a new one only if the private key may have leaked)
- **Status:** open (issues/human/make-the-backup-key.md)
- **Time:** about 15 minutes
- **Cost:** $0

## What it's for

The nightly backup job encrypts every D1 export with
[age](https://age-encryption.org) before uploading it, because the repo is
public and so are its workflow artifacts
([decision 0030](../decisions/0030-backups-and-disaster-recovery.md),
part 3). The public key encrypts; only the private key decrypts. Without
it, backups phase 1 can't start.

## Why a person

The private key is only useful in a disaster if it exists outside GitHub
and Cloudflare, so a person has to see it and keep a copy. Infra Mint
pipes every secret it makes straight into GitHub, where no one sees it.

## Before you start

- age installed on your own computer: `brew install age` (macOS),
  `winget install FiloSottile.age` (Windows) or your Linux package
  manager.
- Your password manager open.
- The `backups` GitHub environment, which backups phase 1 creates. If it
  doesn't exist yet, do steps 1 to 3 now and step 4 later.

## Steps

1. In a terminal: `age-keygen -o backups.age.key`. It prints
   `Public key: age1...`.
2. Copy the whole file's contents (the line starting `AGE-SECRET-KEY-`)
   into a new password manager entry named "jimhillr backup key", and
   print or write it on the same sheet as your recovery codes.
3. Send the **public key** (`age1...`, safe to share) to any thread and
   ask it to commit it as `backups/keys/backups.age.pub`.
4. github.com/jimhillr/monorepo > Settings > Environments > `backups` >
   Add environment secret: name `BACKUP_AGE_KEY`, value the
   `AGE-SECRET-KEY-` line.
5. Delete `backups.age.key` from your computer (empty the trash too).

## Where secrets go

The private key: your password manager, the paper sheet, and the `backups`
environment secret `BACKUP_AGE_KEY` (for the monthly drill). Never in git,
OpenTofu state, a chat or a note. The public key goes in git.

## Verify

The next monthly drill (`backups-drill.yml`) decrypts the newest export
and passes. Tell the thread "done" so it deletes the issue.

## Undo or rotate

Make a new pair with the same steps, commit the new public key, replace
`BACKUP_AGE_KEY`, and keep the old private key until the last export
encrypted to it has expired (90 days). Add a row for it in
[secret-rotation.md](../secret-rotation.md) when phase 1 lands.
