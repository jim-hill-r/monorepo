---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Review dashboard epic / Phase 3: live GitHub data"
order: 110
added: 2026-11-26
---

Put pane behind Cloudflare Access with GitHub as its login method, allowing an Access group of internal people (Jim first) that Jim keeps by hand, since emails can't be in the public repo, and make every `/api/*` route verify the `Cf-Access-Jwt-Assertion` JWT against the Access team's keys and fail closed. Add the human issue files for creating the Access application and policy, and for any deploy-token permission the Worker deploy needs. This is also phase 2 of the deck sign-off epic ([plan](../docs/plans/2026-11-deck-sign-off.md)): internal tools use Access, products keep Auth0.
