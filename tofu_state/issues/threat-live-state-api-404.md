---
kind: fix
priority: priority
tags: [agent-generated]
epic: Threat check
added: 2026-11-27
---

The weekly live threat check (`all-free-tier-usage.yml`, [run 46074536751](https://github.com/jimhillr/monorepo/actions/runs/46074536751)) found surface `state API` in `docs/threat.toml`, at https://jimhillr-tofu-state.jim-hill-r.workers.dev, answering 404 to a request without a token, where the threat model (`access = "token"`, TOFU-T1) expects 401 or 403. Find out whether the Worker isn't served at that workers.dev URL (route, `workers_dev`, the account subdomain) or answers 404 before its auth check on `/`, then fix the Worker or the surface's `url` so an unauthenticated request is refused with 401 or 403.
