---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Resource cleanup / Phase 5: everything onboarded to IaC"
order: 20
added: 2026-11-26
---

Resource cleanup phase 5 (see the [plan](../plans/2026-11-resource-cleanup.md), Part 6). Make the sweep read the GitHub repo settings with `GITHUB_TOKEN` (general settings, labels, Actions permissions, environments, Dependabot settings, installed apps) and write them sorted to `standards/live/github.json`, reporting every setting as unmanaged until it is in IaC. Settings the token can't read (ruleset details, secret and variable names, webhooks, deploy keys, code security) are read by the digest's agent step with its own GitHub access. Never write a secret value.
