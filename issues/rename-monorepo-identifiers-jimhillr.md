---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

Rename the remaining `monorepo` identifiers to `jimhillr` (AGENTS.md naming rule, Jim, 2026-11-27): the `monorepo/` workspace folder and its `workflow_tests` crate (and the CI paths and docs that name it); the `infra` github stack's `github_repository.monorepo` addresses, with OpenTofu `moved` blocks so nothing is recreated (`infra/src/stacks/github.rs`, `infra/src/guard.rs`, the plan fixtures); and the token and key labels `monorepo-billing-read` and `monorepo-replay-read`, at each one's next rotation, updating `standards/settings.toml` and the runbooks in the same PR. Leave the GitHub repository name and mise's `monorepo_root` setting alone.
