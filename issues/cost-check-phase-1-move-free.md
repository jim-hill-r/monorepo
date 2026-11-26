---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Cost check
order: 20
added: 2026-11-26
---

Cost check phase 1. Move the free limits and budgets from `standards/docs/free-tiers.md` into `standards/radar/limits.toml` (`key`, `blip`, `free`, `budget`, `scope`, `unit`, `source`, `checked`), make `free-tiers.sh` read its budgets from it, and generate the free-tiers.md limit tables from it with a check in `standards` CI that they match.
