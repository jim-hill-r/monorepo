---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Feedback on every change
order: 10
added: 2026-11-26
---

Preview deploys. Add a `preview` task to `standards/tasks/cloudflare-pages.toml` (`wrangler pages deploy --branch pr-<n>`), run it in PR CI for affected deploy projects on same-repo PRs, and keep one sticky PR comment with the preview URLs.
