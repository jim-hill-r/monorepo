---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Deck sign-off / Phase 2: Auth0 for pane"
order: 20
added: 2026-11-28
---

Add pane to the infra auth0 stack: a single-page-app client for pane's Worker URL, an Auth0 API for pane with one permission, `review:decks`, a `deck-reviewer` role carrying it, and an Action that adds the reviewer's public handle from `app_metadata.reviewer` to the access token as the `https://jimhillr-pane/reviewer` claim. This is a credential change (Jim's second approval, [decision 0010](../../standards/docs/decisions/0010-credential-second-approval.md)), so label the PR `credential-change`. Add a human task and runbook for giving someone the role and their handle in the Auth0 dashboard, and the `standards/settings.toml` entry for the reviewer list. See the [plan](../docs/plans/2026-11-deck-sign-off.md).
