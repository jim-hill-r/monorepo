---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Deck sign-off / Phase 3: sign-off form"
order: 30
added: 2026-11-28
---

Add the sign-off form to pane's Review page: verdict, feedback items with an optional slide number, and a backlog box. Validate with `pane_model::signoff::files` in the browser before sending, show the files it will write, and send them to `POST /api/reviews/<folder>/signoff` (same origin, so Cloudflare Access's cookie goes with it; pane has no sign-in code of its own). Show the route's errors plainly, and Playwright tests with the route stubbed. See the [plan](../docs/plans/2026-11-deck-sign-off.md).
