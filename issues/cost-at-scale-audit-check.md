---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Cost at scale
order: 10
added: 2026-11-28
---

Add an audit check that every plan and decision record (Markdown in a `plans/` or `decisions/` folder, not a README or fixture) has a `## Cost at scale` section whose table has Now, 5x and 100x rows, as "Cost at scale" in standards/docs/cost.md asks (Jim, 2026-11-28). PR-001 already warns on the PR description; this checks the file itself. Write it as a `chores` command (no new shell scripts), run it in `mise run //:audit`, and start it with a shrinking baseline file in `standards/` listing the plans and decisions merged before the rule, like `standards/user-docs.toml`, so it fails only new files that skip the section.
