---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Feedback on every change
order: 30
added: 2026-11-26
---

Run the smoke tests against each PR preview (production after each deploy already runs them through `standards/scripts/evals.sh smoke`), publish the Playwright HTML report to a `uat-reports` Cloudflare Pages project, and link it from the sticky PR comment.
