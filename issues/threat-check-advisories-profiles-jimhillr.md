---
kind: fix
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 19
added: 2026-11-26
---

Dependency advisories in `profiles/jimhillr`. `cargo update -p tokio` to clear the unsound tokio (RUSTSEC-2026-0386). paste, pin-utils and chrono are flagged unmaintained (RUSTSEC-2024-0436, -2026-0441, -2026-0543) through perseus 0.4.2, which is pinned (AGENTS.md), so add dated ignores with that reason to a profiles/jimhillr `deny.toml`. (SEC-003, found by `bash standards/scripts/deps-check.sh --all`, 2026-11-26.)
