---
kind: fix
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 13
added: 2026-11-26
---

Dependency advisories in `cahokia`. Clear cahokia's advisories: RUSTSEC-2023-0071 (rsa, no fix: propose a dated ignore as for auth_sdk), RUSTSEC-2026-0257 (webbrowser), RUSTSEC-2026-0381 (webpki-roots), RUSTSEC-2026-0387 (crypto-bigint), RUSTSEC-2026-0592 (url) and RUSTSEC-2026-0594 (cookie), plus the yanked spin 0.9.8. Try `cargo update -p <crate>` for each, without moving dioxus off its pinned version (AGENTS.md); for any that need a dioxus bump, add a dated ignore with a reason to a cahokia `deny.toml` instead. (SEC-003, found by `bash standards/scripts/deps-check.sh --all`, 2026-11-26.)
