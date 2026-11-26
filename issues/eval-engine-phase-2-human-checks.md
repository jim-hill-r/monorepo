---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Eval engine
order: 100
added: 2026-11-26
---

Eval engine phase 2. Human checks for settings made by hand (Jim, 2026-11-26; see the plan's "Human checks" section): add a `standards/settings.toml` manifest of hand-configured settings (Claude MCP connectors and scheduled tasks, repo and Worker secrets, Auth0 applications, Cloudflare token scopes and Access policies, PostHog, free-plan billing) with `id`, `vendor`, `where`, `expect`, `why` and `every`, never a secret's value; add the `human-check` eval kind (`setting = "<id>"`, stage `human`) to `evals/model` and `evals check`; list due checks in `evals verdicts`; teach the `human-issues-digest` skill to add each due check to `issues/human/` as a periodic human task file (`Check <setting>: ...`) and to record Jim's "ok" or "not ok" answer as a `by = "jim"` result through `evals publish` (not ok files a `FIX`); and document the kind in `standards/docs/evals.md`.
