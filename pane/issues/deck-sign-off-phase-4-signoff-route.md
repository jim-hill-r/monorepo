---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Deck sign-off / Phase 4: sign-off route"
order: 40
added: 2026-11-28
---

Add `POST /api/reviews/<folder>/signoff` to the pane Worker: verify the `Cf-Access-Jwt-Assertion` token against the Access team's public keys (issuer, the pane application's audience, expiry) and fail closed, map its email to a public handle through the `PANE_REVIEWERS` Worker secret (refuse anyone without one, and never write the email anywhere), build the files with `pane_model::signoff::files` from the deck in the deployed snapshot (refusing an unknown deck or a second sign-off by the same reviewer that day), and commit them to the open `pane-edit` PR through the `/api/edits` write path, with the handle in the commit message. Add a human task and runbook for setting `PANE_REVIEWERS`, its `standards/settings.toml` entry, and the route's threats in `pane/docs/threat.toml`. Needs phase 2 (Access, order 110), the pane Worker (merged in #759) and `api-edits-route-pane-worker-inserts.md`. See the [plan](../docs/plans/2026-11-deck-sign-off.md).
