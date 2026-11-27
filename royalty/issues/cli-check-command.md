---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Royalty / Phase 1: MVP scanner and report"
order: 40
added: 2026-11-27
---

Add the `royalty_cli` crate (`cli/`) with the `royalty` binary and its first command, `royalty check [path]`: parse and validate a `royalty.toml`, print each finding with the `ROYL-nnn` requirement it relates to, exit 0 on warnings and 1 on errors, and treat warnings as errors with `--strict`. Document the command in `README.md`.

Blocked on: decision 11 in the plan.
