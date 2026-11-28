---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Baked skills / Phase 2: more commands"
order: 10
added: 2026-11-27
---

Add `chores credit-window`: given the newest `rate_limit_event`s (JSON from the session API) and the counts of working and open threads, apply the rule in decision 0013 (stop, burn rate, projection, weekly guard, burst, ease off, floor) and print how many threads to start and the numbers behind it, so the hourly credit-window check is one call. Wait for [PR 819](https://github.com/jimhillr/monorepo/pull/819) to merge (it adds the decision), then wire it into [the scheduled task prompt](../../../standards/docs/scheduled-tasks/). See the [baked skills plan](../../../standards/docs/plans/2026-11-baked-skills.md).
