---
kind: fix
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 14
added: 2026-11-26
---

Dependency advisory in `controlled_chaos`: RUSTSEC-2026-0344 (ruzstd 0.7 decompression bomb, fixed in 0.9.1). It comes through `bevy_render` 0.14, so it clears with a bevy upgrade, not a lockfile update. Do it with the next bevy bump, or add a dated ignore with a reason to a `controlled_chaos/deny.toml` (the game only decompresses its own bundled assets) in a PR labelled `needs-human`. The yanked async-executor was updated on 2026-11-26. (SEC-003, `bash standards/scripts/deps-check.sh --all`.)
