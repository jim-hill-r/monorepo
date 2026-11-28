---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Project isolation
order: 90
added: 2026-11-28
---

Once every project's write-isolation-guide.md issue is closed and `chores isolation` reports nothing, switch the ISO-* step in standards/scripts/audit.sh to `chores isolation --strict` so a missing or stale guide fails the audit, and drop "warnings only" from standards/docs/project-isolation.md and chores/README.md (decision 0026). Not before: it would turn the audit red for every project at once.
