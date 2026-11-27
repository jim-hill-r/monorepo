---
kind: fix
priority: priority
tags: [agent-generated]
epic: Threat check
added: 2026-11-27
---

The weekly live threat check (`all-free-tier-usage.yml`, [run 46074536751](https://github.com/jimhillr/monorepo/actions/runs/46074536751)) found surface `event API` in `docs/threat.toml`, at https://jimhillr-telemetry.jim-hill-r.workers.dev, answering 404. Find out whether the Worker isn't served at that workers.dev URL any more (route, `workers_dev`, the account subdomain) or the check asks a path the Worker doesn't answer, then fix the Worker, its `wrangler.toml` or the surface's `url` so the check gets the answer the threat model expects.
