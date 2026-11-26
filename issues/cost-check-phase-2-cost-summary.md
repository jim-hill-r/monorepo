---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Cost check
order: 70
added: 2026-11-26
---

Cost check phase 2. Add `cost summary --base <ref>` and a PR CI step that writes the cost summary to the job summary and sets the `cost` label, plus `needs-human` when a PR raises money, adds a vendor to a project or raises a budget. Make `hooks.sh` run `cost check --projects <affected>` after `//:audit`, with cases in `standards/tests/hooks.test.sh`.
