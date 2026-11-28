---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Deck sign-off / Phase 3: sign-in and form"
order: 30
added: 2026-11-28
---

Sign in to pane through `auth_sdk`'s `WebAuthProvider` with the Auth0 client from phase 2 (reading pane stays open; only signing off needs sign-in), and add the sign-off form to the Review page: verdict, feedback items with an optional slide number, and a backlog box. Validate with `pane_model::signoff::files` in the browser before sending, show the files it will write, and send them to `POST /api/reviews/<folder>/signoff` with the access token. Playwright tests with a stubbed sign-in. See the [plan](../docs/plans/2026-11-deck-sign-off.md).
