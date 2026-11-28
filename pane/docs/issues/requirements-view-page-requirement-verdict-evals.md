---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Review dashboard epic / Eval engine: requirements and feedback"
order: 250
added: 2026-11-26
---

Add a Requirements view to each Project page: each requirement's verdict (from `evals/model`), each eval's latest result with its evidence link, and its history, read into the snapshot at build time from the `evals-results` branch (`verdicts.json` and `results/`), and redeploy pane after each eval run. Add a Requirements signal (worst `must` verdict, with requirement coverage) to the Health page.
