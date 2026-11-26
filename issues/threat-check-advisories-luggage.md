---
kind: fix
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 17
added: 2026-11-26
---

Dependency advisories in `luggage`. Clear luggage's advisories: RUSTSEC-2026-0452 to -0454 (surrealkv, surrealdb, surrealdb-core), -0548 (zip), -0204 (crossbeam-epoch), -0381 (webpki-roots), -0592 (url), -0594 (cookie), the unsound tokio (RUSTSEC-2026-0386) and the yanked spin 0.9.8. luggage is on hold, so a dated ignore list in a luggage `deny.toml` is fine if updating surrealdb is too big. (SEC-003, found by `bash standards/scripts/deps-check.sh --all`, 2026-11-26.)
