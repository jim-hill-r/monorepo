---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Product promotion / Phase 2: chores promotion"
order: 20
added: 2026-11-28
---

Add `chores promotion --project <name>`: report each agent-checkable promotion gate (G1, G2, G5 to G13 in [product lifecycle](../standards/docs/product-lifecycle.md#promotion-gates)) as Met or Not met with its evidence, reading the existing checks' outputs (product-docs, cost check, evals results, threat check, user-docs, site-check, resources) from files, never the network. Print the Promotion scorecard table the deck uses, and with `--issues` write one issue file per unmet gate in the product's own issues folder. Wire it into the deep review skill's promotion mode.
