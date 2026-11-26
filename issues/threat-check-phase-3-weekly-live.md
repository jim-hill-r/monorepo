---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 110
added: 2026-11-26
---

Threat check phase 3. Add the weekly live check to `all-free-tier-usage.yml`: list every Pages project and Worker and file a `FIX` for any no `threat.toml` claims (such as `pane-dbc.pages.dev`), and probe each declared URL and one preview URL per Pages project for its expected access (Cloudflare Access redirect, `401` without a token) and the required headers.
