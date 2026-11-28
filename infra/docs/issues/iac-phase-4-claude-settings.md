---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Infrastructure as code / Phase 4: onboard everything"
order: 90
added: 2026-11-26
---

Decision 0007: `infra` also synthesizes `infra/out/claude.json` (project MCP connectors and that they are read-only, the cloud environment's allowed network hosts and setup script, scheduled tasks and their schedules). Add a `claude-settings` agent skill that compares it with what the Claude tools report and reports drift in the weekly human issues digest; agents apply changes only after the PR changing `claude.json` is approved, and connectors stay Jim's to change.
