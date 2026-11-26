---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-26
---

Add a `--json` flag that prints the property details and the evaluation (`criteria_checked`, `criteria_met`, `meets_preferences`, `unmet_criteria`) as one JSON object on stdout instead of the text report, so homer's output can be scripted and checked by evals (see the root `../../issues/eval-engine-phase-2-trycmd-script.md`) without parsing text. Derive `Serialize` for `PropertyEvaluation`, add a trycmd transcript for `--json`, and document the flag in `../README.md`.
