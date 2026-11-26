---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Review dashboard epic / Phase 3: live GitHub data"
order: 110
added: 2026-11-26
---

Put pane behind Cloudflare Access allowing only Jim, and make every `/api/*` route verify the `Cf-Access-Jwt-Assertion` JWT against the Access team's keys and fail closed. Add the human issue files for creating the Access application and policy, and for any deploy-token permission the Worker deploy needs.
