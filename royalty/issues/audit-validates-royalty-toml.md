---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Royalty / Phase 1: MVP scanner and report"
order: 90
added: 2026-11-27
---

Run `royalty check` on every project's `royalty.toml` from `mise run //:audit` through a `standards/scripts/royalty-check.sh`, as warnings first, in the way `cost-check.sh` runs `cost check`. Add a finding for a project with no `royalty.toml`, which `standards/docs/configuration.md` requires, and document the codes there.

Blocked on: decision 11 in the plan.
