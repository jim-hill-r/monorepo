---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Cost tracking / Phase 2: Claude Code and the digest"
order: 20
added: 2026-11-27
---

Cost tracking phase 2 (standards/docs/plans/2026-11-cost-tracking.md). Have PR triage read `rate_limit_info.isUsingOverage` from the session API each run and report it as red to Jim; the coordinator then starts no new threads until usage resets (AGENTS.md "Pause on Claude overage"). Add a short Cost section to the `human-issues-digest` skill (money this month, every yellow or red limit from the latest usage run, any overrun switch whose check is due), and add the "Overrun switches" table to `standards/docs/free-tiers.md`.
