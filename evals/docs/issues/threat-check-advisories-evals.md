---
kind: fix
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 15
added: 2026-11-26
---

Dependency advisories in `evals`. Chrono is flagged unmaintained (RUSTSEC-2026-0543). Replace it with `jiff` or `time` in evals, or add a dated ignore with a reason to an evals `deny.toml`. (SEC-003, found by `bash standards/scripts/deps-check.sh --all`, 2026-11-26.)
