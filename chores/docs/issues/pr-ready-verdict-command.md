---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Baked skills / Phase 2: more commands"
order: 30
added: 2026-11-27
---

Add `chores pr-ready`: given a PR's check runs and the latest `main` commit's check runs (JSON the agent fetched with its GitHub tools), print green, pending or red under the merge rule in [pull-requests.md](../../../standards/docs/pull-requests.md) (a check also failing on `main` doesn't count), and the next step (wait, fix which check, or mark ready and turn on auto-merge). Use it from `open-pull-request`. See the [baked skills plan](../../../standards/docs/plans/2026-11-baked-skills.md).
