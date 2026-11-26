---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Eval engine
order: 60
added: 2026-11-26
---

Eval engine phase 2. Run `evals run --stage cd` after `deploy` and `smoke` in `mise-projects.yml` against `DEPLOY_URL`, failing the deploy job on a failed `must` eval; add the `http` eval kind (status, header, body and response time checks).
