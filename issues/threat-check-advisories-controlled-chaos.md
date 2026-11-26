---
kind: fix
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 14
added: 2026-11-26
---

Dependency advisories in `controlled_chaos`. `cargo update -p ruzstd -p async-executor` in controlled_chaos to clear RUSTSEC-2026-0344 (ruzstd decompression bomb) and the yanked async-executor 1.14.0; if ruzstd is held back by bevy, record why in a dated ignore in a controlled_chaos `deny.toml`. (SEC-003, found by `bash standards/scripts/deps-check.sh --all`, 2026-11-26.)
