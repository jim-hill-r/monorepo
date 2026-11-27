---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Infrastructure as code / Phase 4: onboard everything"
order: 80
added: 2026-11-26
---

Decision 0007, "How it fits the cleanup plan": the resource cleanup sweep (`resources.sh sweep`) reads the synthesized `infra/out/*.tf.json` as a claim source and reports "live but not managed by IaC" as its own finding, unless a `[[keep]]` entry in `standards/resources.toml` gives a reason. It also lists GitHub repo settings, rulesets, labels, environments and secret names, so unwritten settings show up. Settings that move into IaC leave the eval engine's `standards/settings.toml` human checks (or get `managed = "infra"`). Needs resource cleanup phase 1.
