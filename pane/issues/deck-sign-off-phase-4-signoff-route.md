---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Deck sign-off / Phase 4: sign-off route"
order: 40
added: 2026-11-28
---

Add `POST /api/reviews/<folder>/signoff` to the pane Worker: verify the Auth0 access token against Auth0's public keys (issuer, audience, expiry, the `review:decks` permission and the reviewer claim) and fail closed, build the files with `pane_model::signoff::files` from the deck in the deployed snapshot (refusing an unknown deck or a second sign-off by the same reviewer that day), and commit them to the open `pane-edit` PR through the `/api/edits` write path, with the reviewer handle in the commit message. Needs the pane Worker (PR #759) and `api-edits-route-pane-worker-inserts.md` with its token. Add the route's threats to `pane/docs/threat.toml`. See the [plan](../docs/plans/2026-11-deck-sign-off.md).
