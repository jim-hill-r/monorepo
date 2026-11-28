---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Code quality / Phase 3: Loop"
order: 50
added: 2026-11-27
---

Add `quality-triage.yml` (weekly, standard runners): run `chores quality --report` over every project and write at most 10 `quality-<project>-<slug>.md` issue files for the worst findings in projects with no open `quality-*` file, force-push them to the `quality-triage` branch (its only push) for PR triage to open as a draft, the same way `evals-triage.yml` works. Add a dry-run input. Document it in `code-quality.md` and `standards/docs/scheduled-tasks/pr-triage.md`. Plan: [standards/docs/plans/2026-11-code-quality.md](../plans/2026-11-code-quality.md).
