---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 22
added: 2026-11-26
---

Threat check phase 1 follow-up. Turn on cargo-deny's licence check in `standards/scripts/deps-check.sh` (the `[licenses]` allowlist is already in `standards/deny.toml`). Our own crates fail it as unlicensed today, so first give every crate `publish = false` (or the repo's licence), then fix or clarify any third-party licence outside the allowlist (surrealdb's BSL in luggage is one).
