---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Review roles / Phase 2: all domains"
order: 20
added: 2026-11-28
---

Build phase 2 of [decision 0019](../standards/docs/decisions/0019-review-roles.md): add the `change`, `threat`, `quality`, `credential`, `minting` and `docs` (the `docs-only` tag, `docs-reviewer`) domains, and the `observability` domain with its `observability-reviewer` grant and path trigger from [decision 0023](../standards/docs/decisions/0023-observability.md), to the check, rename the `Human review` check and workflow to `Review roles` (and the required check in `infra`'s `github` stack), and make `standards/scripts/auto-review.sh` follow the check instead of its own label, tag and path lists (bump its rules version). After this, `jimhillr-reviewer` approves plain product changes only.
