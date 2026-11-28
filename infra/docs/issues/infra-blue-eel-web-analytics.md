---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

`infra/src/stacks/mod.rs` lists Blue Eel (`sparkhill_webapp`, Pages project `jimhillr-sparkhill-webapp`) without Web Analytics, unlike every other web app. Turn it on with `.web_analytics()`, run `mise run synth`, and check the PR plan creates one `cloudflare_web_analytics_site` and nothing else.
