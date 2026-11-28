---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Resource cleanup / Phase 5: everything onboarded to IaC"
order: 10
added: 2026-11-26
---

Resource cleanup phase 5 (see the [plan](../plans/2026-11-resource-cleanup.md), Part 6). Add the `[iac]` table (`managed` kinds and `[[iac.exempt]]` with reasons and review dates) to `standards/resources.toml`, and give the sweep the **unmanaged** (live but not in IaC) and **drift** (live settings differ from IaC, read from decision 0007's weekly drift plan in the same run) findings: warnings for every kind, errors for kinds in `managed`. Extend RES-001 so removing an IaC definition needs a retirement entry.
