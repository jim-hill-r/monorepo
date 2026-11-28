---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Backups / Phase 3: drills"
order: 30
added: 2026-11-28
---

Add `.github/workflows/backups-drill.yml`, monthly ([decision 0030](../../standards/docs/decisions/0030-backups-and-disaster-recovery.md), part 5): fetch `repo/latest` from KV, `git bundle verify`, clone, check `main` is under 24 hours behind GitHub and `cookbook/content/` has as many files as GitHub's `main`; download the newest D1 artifact, decrypt it with `BACKUP_AGE_KEY`, load it into SQLite, run `PRAGMA integrity_check` and a row count. A failure opens an observability incident (decision 0023) once that exists, else an issue file. Record results as the BACK evals' production evidence.
