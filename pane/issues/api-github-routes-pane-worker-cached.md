---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Review dashboard epic / Phase 3: live GitHub data"
order: 120
added: 2026-11-26
---

Add `/api/github/*` routes to the pane Worker, cached for 5 minutes with the Cache API: open PRs (draft, ready, `needs-human`, age, checks, projects touched), latest workflow runs per project and per system workflow, releases with asset download URLs, and production deployments. Add the human issue file for the GitHub token secret (read scope is enough until Phase 5).
