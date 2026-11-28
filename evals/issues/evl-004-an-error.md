---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Evals roadmap / Phase 2: Cover every project"
order: 60
added: 2026-11-26
---

Make EVL-004 (a `must` requirement with no production-stage eval) an error in `evals check`. Blocked: it would fail the audit today, since 16 musts have no production eval (`evals check`, 2026-11-28: DSGN-002, LANG-001 to 003, PANE-013, ROYL-001 to 009, TELE-004, TOFU-003). Do it once the Phase 2 issues of the [evals roadmap](../docs/roadmap.md) that give them one have landed and `evals check` shows no EVL-004 finding; then fail on EVL-004 only, in `evals check` and `standards/scripts/evals-check.sh`, and update "Checks" in `standards/docs/evals.md`.
