# backups user guide

## Who it's for

Jim, who keeps the offline copies and the recovery codes and decides when
to restore, and the agents who run a restore. It answers "if this is lost,
how do we get it back, and how much do we lose?"

Today this is planned work (decision 0030); the steps below describe how
it will work once phases 1 and 2 ship.

## Getting started

1. Read [the inventory](../inventory.md) to see what is backed up, how
   often, and what is deliberately not.
2. Do the two one-time tasks the weekly digest brings you: save your
   account recovery codes, and make the backup key.
3. When the 90-day settings check "Offline copy" comes up, download the
   latest backup to your own computer and answer the check.

## How to ...

### Get back a file or a deleted branch

Ask an agent. Git history usually has it; if not, the agent restores it
from the newest nightly bundle.

### Undo a bad change to D1 data

Ask an agent to restore the database with D1 Time Travel to a minute
before the change (up to 7 days back), or from the nightly encrypted
export (up to 90 days back), following
[disaster-recovery.md](../../../standards/docs/runbooks/disaster-recovery.md).

### Recover after losing GitHub or Cloudflare

Follow [disaster-recovery.md](../../../standards/docs/runbooks/disaster-recovery.md).
It starts from the copy that is left (Cloudflare KV, or your offline copy)
and lists the steps only you can do.

### Read the cookbook with nothing else left

Unzip your offline copy and open `cookbook/content/`: every recipe is a
Markdown file you can read in any text editor.

## Getting help

- A drill failed: the incident issue says which check; ask an agent to
  fix it.
- You can't find your recovery codes or backup key: tell an agent at once,
  so it can help you make new ones while you can still sign in.
- Something else: ask in the project chat.
