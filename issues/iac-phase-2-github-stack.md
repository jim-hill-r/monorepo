---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Infrastructure as code / Phase 2: GitHub"
order: 60
added: 2026-11-26
---

Decision 0007, phase 2: a `github` stack in `infra` that imports this repo and manages delete head branches on merge, the `main` ruleset (with the `CI result` required check, which replaces that human issue), labels such as `needs-human`, and the `infra` environment. First add a human issue for a fine-grained token with Administration on this repo only (read for plans, write for apply) and a yearly rotation check.
