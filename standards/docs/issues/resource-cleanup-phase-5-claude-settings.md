---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Resource cleanup / Phase 5: everything onboarded to IaC"
order: 30
added: 2026-11-26
---

Resource cleanup phase 5 (see the [plan](../plans/2026-11-resource-cleanup.md), Part 6). Give the `human-issues-digest` skill a step that reads Claude scheduled tasks and cloud environments (network policy, setup script, environment variable names, never values) with its Claude tools, writes them sorted to `standards/live/claude.json` in a PR, and reports any that are unmanaged or drifted. Add `human-check` entries in `standards/settings.toml` for what no tool can read (project instructions, connectors and plugins).
