---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

Turn DOC-001 to DOC-004 and LIN-001/LIN-002 in `standards/scripts/project-check.sh` from warnings into errors once every project passes them. On 2026-11-27 about 35 DOC warnings (mostly missing CONTRIBUTING.md files and README.md headings that don't name the project) and two LIN warnings (`my_lang`, fixed by PRs #786 and #787) remained. Fix the remaining projects one PR per project (skip the on hold ones until they come back, or exempt them), then switch `warn` to `err` for each check and update `standards/docs/documentation.md` and `rust.md`.
