---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 60
added: 2026-11-26
---

Threat check phase 2. Add the `threat` Rust project (`threat/model`, `threat/cli`) with `threat check` for THR-001 to THR-003 and THR-005 to THR-009 as in the plan, unit tests for each rule, and `standards/scripts/threat-check.sh` called from `//:audit` as warnings only (the `evals-check.sh` pattern).
