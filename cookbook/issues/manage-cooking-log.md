---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Management app / Phase 5: the cooking log"
order: 50
added: 2026-11-28
---

Let a curator log "we cooked this" for a recipe in the management app: the date, who cooked (a display name the curator picks, never an email), a rating, notes and what they changed, and see every past cook of the recipe. Store the log in the recipe's own Markdown file (a `## Cooking log` section in `docs/content-format.md`, parsed and written by `data_md`), saved through [content PRs](manage-content-prs.md); the notes are public like the recipes (Jim, 2026-11-28: "I don't think we need private notes"). Logging a cook can move a recipe from `draft` to `tested`. See the [roadmap](../docs/roadmap.md).
