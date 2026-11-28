---
kind: fix
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 17
added: 2026-11-28
---

Dependency advisories left in `luggage` after the surrealdb 4 upgrade (2026-11-28), which cleared all 31 of its Dependabot alerts. surrealdb 4 brings in rsa 0.9 (RUSTSEC-2023-0071, Marvin timing attack, no fixed 0.9 release) and crypto-bigint 0.5 (RUSTSEC-2026-0387), and some crates flagged unmaintained: heapless 0.6 to 0.8 (RUSTSEC-2026-0574 to -0582), atomic-polyfill (RUSTSEC-2023-0089), bincode 2 (RUSTSEC-2025-0141), humantime (RUSTSEC-2026-0542) and chrono (RUSTSEC-2026-0543). They clear with a later surrealdb release; luggage is on hold, so a dated ignore list with that reason in a luggage `deny.toml` is fine meanwhile. (SEC-003, `cargo audit` in luggage, 2026-11-28.)
