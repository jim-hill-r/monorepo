---
kind: fix
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 18
added: 2026-11-26
---

Dependency advisories in `pane`. `cargo update -p url` in pane to clear RUSTSEC-2026-0592 (url 2.5.8, fixed in 2.5.9). (SEC-003, found by `bash standards/scripts/deps-check.sh --all`, 2026-11-26.)
