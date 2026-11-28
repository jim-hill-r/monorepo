---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Project isolation
added: 2026-11-28
---

Write `infra/docs/isolation.md` so infra could move to another repo or be sold and still mostly work (Jim, 2026-11-28; decision 0026, standards/docs/project-isolation.md). `## Moving out` lists what a copy of the folder needs outside this repo: shared mise tasks, workflows, path dependencies, secrets, badges and data branches. The check found no shared live resource in its files, so `## Shared resources` can say `None.` once you have confirmed that. It owns `cloudflare-account` and `auth0-tenant`: under `## Shared resources` say so and list the projects that use them (run `chores isolation --json`), with a pointer to each user's own guide. Done when `chores isolation` reports nothing for infra. telemetry/docs/isolation.md is the example.
