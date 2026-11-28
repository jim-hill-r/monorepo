---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Royalty / Phase 1: MVP scanner and report"
order: 95
added: 2026-11-27
---

Add a `royalty.toml` with a `[project]` `name` and `description` to every project the audit's ROY-001 lists (`bash standards/scripts/royalty-check.sh`; 24 on 2026-11-27, such as `cost`, `evals`, `threat` and the parent folders of `cahokia`, `cookbook` and `pane`), as `standards/docs/configuration.md` requires. Use the name and one-sentence description from each project's README. Then make ROY-001 and RTOML-* errors in `royalty-check.sh` and say so in `configuration.md`.
