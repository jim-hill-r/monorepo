---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Management app / Phase 4: saving as content PRs"
order: 40
added: 2026-11-28
---

Let the management app's server (after [the server check](manage-api-token-check.md)) save a curator's change as a commit to a content PR that changes only `cookbook/content/`, one open PR at a time that later saves add to, through a GitHub App installation token minted by Infra Mint (`infra/scripts/mint.sh`, a credential change under decision 0010, with its row in `standards/docs/secret-rotation.md`). Validate every file with `data_md` before committing. Add a case to the reviewer account's written rules ("The reviewer account" in `standards/docs/pull-requests.md`) so a green content-only PR from the management app is approved, which is a protected-path change for Jim. Record the decision in the [roadmap](../roadmap.md) (Jim, 2026-11-28: option A).
