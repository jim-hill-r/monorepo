# Contributing to backups

## Getting Started

This is a docs-only project until phase 1 adds the nightly D1 export job, so
there is nothing to build yet:

```bash
cd backups
mise run ci   # docs-only project, nothing to check
```

Pick up the next phase from [issues/](issues/) with the `start-a-new-task`
skill. Read [decision 0030](../standards/docs/decisions/0030-backups-and-disaster-recovery.md)
first. Three rules matter most:

- The repo is public, so anything a workflow uploads can be downloaded by
  anyone. Encrypt every export of data that isn't already in the repo
  before it leaves the runner.
- A backup that has never been restored doesn't count: every new backup
  gets a check in the drill.
- Adding a stateful resource anywhere in the repo means adding its row to
  [docs/inventory.md](docs/inventory.md).
