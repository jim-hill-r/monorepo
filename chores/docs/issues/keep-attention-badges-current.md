---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Baked skills / Phase 2: more commands"
order: 40
added: 2026-11-27
---

Keep the PR attention tags current after later pushes: a PR CI step runs `chores badges --base origin/main` and, when the `## Attention` section in the description differs, replaces it (the same way the Security summary job sets labels). Needs `pull-requests: write` on that job, which is itself a risk change for Jim to approve. Until then agents rerun it by hand ([open-pull-request](../../../.agents/skills/open-pull-request/SKILL.md)). See "Attention tags" in [pull-requests.md](../../../standards/docs/pull-requests.md).
