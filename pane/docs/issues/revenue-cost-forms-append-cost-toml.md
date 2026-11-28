---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Review dashboard epic / Phase 7: revenue and cost"
order: 350
added: 2026-11-26
---

Add "Add revenue" and "Add a cost" forms that append a `[[revenue]]` or `[[cost]]` entry to the project's `docs/cost.toml` through the `pane-edit` PR (which `cost check` then verifies), and name a `docs/cost.toml` change as safe for PR triage to approve in `standards/docs/pull-requests.md` only when it is a `pane-edit` PR that adds revenue or a $0 cost; any cost above $0 stays `needs-human`.
