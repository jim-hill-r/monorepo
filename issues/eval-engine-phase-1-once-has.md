---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Eval engine
order: 10
added: 2026-11-26
---

Eval engine phase 1. Once a project has `docs/requirements.md` or `evals.toml`, the audit in the `plan` job of `mise-projects.yml` builds the evals CLI on every run (about a minute cold). Add a `Swatinem/rust-cache` step for `evals` there (saved from main only), in a PR whose CI can run every project.
