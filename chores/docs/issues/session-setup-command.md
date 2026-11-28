---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Baked skills / Phase 2: more commands"
order: 20
added: 2026-11-27
---

Bake cloud-sandbox session setup into one step: install mise from npm when it is missing, run the hooks setup, and fall back to `MISE_OFFLINE=1` as [git-hooks.md](../../../standards/docs/git-hooks.md) describes, printing only what failed. It runs before cargo can build `chores`, so it may stay a shell script in `standards/scripts/` wrapped by a `session-setup` skill. See the [baked skills plan](../../../standards/docs/plans/2026-11-baked-skills.md).
