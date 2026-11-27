---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Royalty / Phase 1: MVP scanner and report"
order: 60
added: 2026-11-27
---

Give `royalty scan` its unit counts: seats as the number of distinct commit authors of the consumer's repository in the period from `git log` (bots left out, `[consumer] seats` overrides, emails never written to any output), deployments from `[[consumer.deployment]]` and each one's manifest graph, `project` and `flat` from the graph, and `revenue` and `custom:` units from `[[consumer.package]]` declarations. A unit that cannot be counted is reported as "not measured", never guessed.

Blocked on: decision 11 in the plan.
