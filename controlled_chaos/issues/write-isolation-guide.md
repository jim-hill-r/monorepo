---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Project isolation
added: 2026-11-28
---

Write `controlled_chaos/docs/isolation.md` so controlled_chaos could move to another repo or be sold and still mostly work (Jim, 2026-11-28; decision 0026, standards/docs/project-isolation.md). `## Moving out` lists what a copy of the folder needs outside this repo: shared mise tasks, workflows, path dependencies, secrets, badges and data branches. Its files use the Cloudflare account (`cloudflare-account`), the telemetry Worker (`telemetry-worker`): give each a `###` section with `#### Why it is shared` and `#### Migration guide` (steps, cost after separating, what data moves, how to verify). Done when `chores isolation` reports nothing for controlled_chaos. telemetry/docs/isolation.md is the example.
