---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Eval engine
order: 90
added: 2026-11-26
---

Eval engine phase 2. Make EVL-004 (a `must` requirement with no production-stage eval) an error in `evals check`. Blocked: it would fail the audit today, since my_lang's LANG-001 to LANG-003 have no production eval yet (checked 2026-11-27). Do it once `my_lang/docs/issues/production-evals-with-the-cli.md` lands and `evals check` shows no EVL-004 finding; then fail on EVL-004 only, in `evals check` and `standards/scripts/evals-check.sh`, and update "Checks" in `standards/docs/evals.md`.
