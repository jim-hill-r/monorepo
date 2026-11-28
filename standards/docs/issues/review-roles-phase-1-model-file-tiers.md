---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Review roles / Phase 1: model, file and tiers"
order: 10
added: 2026-11-28
---

Build phase 1 of [decision 0019](../decisions/0019-review-roles.md): add `standards/reviewers.toml` (tiers and grants as the record lists them), a `review_roles` module in `chores` (parse, the fixed rules, the check over domain and project cells and scopes), and make `chores gate` read it for the `requirements` and `cost` domains. Add audit rules REV-001 (the file breaks a fixed rule or names an unknown role or scope) and REV-002 (a top-level folder in no tier or in two). Add the three tiers to AGENTS.md and `REPOSITORY_STRUCTURE.md`, and update "Human review" in `standards/docs/pull-requests.md`. Needs PR 904 merged first.
