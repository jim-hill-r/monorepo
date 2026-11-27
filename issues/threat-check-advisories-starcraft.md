---
kind: fix
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 20
added: 2026-11-26
---

Dependency advisories in `starcraft`. Clear starcraft's advisories: RUSTSEC-2024-0400 and -2026-0344 (ruzstd, also yanked 0.7.2), -2025-0055 (tracing-subscriber), -2026-0194 and -0195 (quick-xml), -0204 (crossbeam-epoch), -0391, -0398, -0403 and -0410 (zstd, zstd-safe, zstd-sys). Most come through bevy; try `cargo update` first, and for the rest record a dated ignore in a starcraft `deny.toml` (starcraft is on hold). (SEC-003, found by `bash standards/scripts/deps-check.sh --all`, 2026-11-26.)
