---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Infrastructure as code / Phase 2: GitHub"
order: 60
added: 2026-11-26
---

Decision 0007, phase 2: the `github` stack in `infra` (`src/stacks/github.rs`) already imports this repo (delete head branches on merge included), code security and a `CI` ruleset. Add the labels the workflows and triage use (`needs-human`, `credential-change`, `cost`) with `github_issue_label` and the `infra` environment with `github_repository_environment` (never `infra-credentials`, IAC-005). Both need more permissions on the read-only infra GitHub App (Issues: read, Environments: read) and the infra GitHub App (Issues: write); update their runbooks and the `github-app-infra*` settings checks. Plans and applies already use the two apps, so no token is needed.
