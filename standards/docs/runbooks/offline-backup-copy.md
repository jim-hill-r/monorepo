# Keep an offline copy of the repo

- **Kind:** recurring, every 90 days (settings check `offline-backup-copy`)
- **Status:** open (the settings check in `standards/settings.toml`)
- **Time:** about 5 minutes
- **Cost:** $0

## What it's for

A copy of the repo, and with it the cookbook content, that survives
losing both GitHub and Cloudflare
([decision 0030](../decisions/0030-backups-and-disaster-recovery.md),
part 4; requirement BACK-002).

## Why a person

The point is a copy outside every account the agents can reach, on your
own computer or drive.

## Before you start

- A folder on your computer or an external drive for backups.

## Steps

1. Until backups phase 1 lands: open github.com/jimhillr/monorepo >
   Code > Download ZIP. This is `main` without history, which is enough
   to read every recipe.
2. Once phase 1 lands: open the newest successful run of the "Backups
   Nightly" workflow under Actions and download its `repo-bundle`
   artifact, which has every branch and the full history.
3. Save it in your backups folder named with today's date, and delete
   copies older than a year.

## Where secrets go

None. The repo is public and holds no secrets.

## Verify

Open the file and check `cookbook/content/` has recipes in it. Answer the
settings check with the date (see [settings-check.md](settings-check.md)).

## Undo or rotate

None.
